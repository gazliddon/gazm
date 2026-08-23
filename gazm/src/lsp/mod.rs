//! Language-server configuration and the transport-independent LSP boundary.
//!
//! The server starts small and transport-focused so document analysis can be
//! added without coupling protocol handling to the assembler internals.

use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex, RwLock};

use grl_sources::{AsmSource, SourceFile};
use unraveler::Severity;

use crate::{
    assembler::Assembler,
    error::ErrorCollectorTrait,
    fmt::format_text,
    frontend::{AstNodeKind, FrontEndErrorKind, TokenizeRequest},
    opts::Opts,
};

mod analysis;
use analysis::ProjectAnalysis;

struct AnalysisJob {
    generation: u64,
    uri: String,
    documents: HashMap<String, String>,
    opts: Opts,
}

fn format_cpu(cpu: crate::cpukind::CpuKind) -> &'static str {
    match cpu {
        crate::cpukind::CpuKind::Cpu6800 => "6800",
        crate::cpukind::CpuKind::Cpu6809 => "6809",
        _ => "unknown",
    }
}

fn target_for_document(uri: &str, opts: &[Opts]) -> usize {
    let path = uri_to_path(uri);
    let canonical = path.canonicalize().unwrap_or(path);
    opts.iter()
        .position(|target| {
            let Some(root) = target.assemble_dir.as_ref() else {
                return false;
            };
            let project = root.join(&target.project_file);
            let project = project.canonicalize().unwrap_or(project);
            canonical == project
                || canonical.parent().is_some_and(|parent| {
                    parent.starts_with(project.parent().unwrap_or(Path::new("")))
                })
        })
        .unwrap_or(0)
}

fn publish_target(writer: &mut impl Write, uri: &str, opts: &Opts) -> Result<(), String> {
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "gazm/target",
        "params": {
            "uri": uri,
            "target": opts.target_name,
            "cpu": format_cpu(opts.cpu)
        }
    });
    write_message(writer, &notification).map_err(|e| e.to_string())
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct LspConfig {
    pub log_file: Option<PathBuf>,
}

pub fn do_lsp(opts: &[Opts]) -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = stdin.lock();
    run_server(&mut input, &mut stdout, opts)
}

fn run_server(
    reader: &mut impl BufRead,
    writer: &mut (impl Write + Send),
    opts: &[Opts],
) -> Result<(), String> {
    let mut documents = HashMap::<String, String>::new();
    let fallback_opts = opts.first().cloned().unwrap_or_default();

    let selected_opts = |uri: &str| -> &Opts {
        opts.get(target_for_document(uri, opts))
            .unwrap_or(&fallback_opts)
    };

    let mut current_generation: u64 = 0;
    let analysis = Arc::new(RwLock::new(ProjectAnalysis::default()));
    let latest_completed = Arc::new(AtomicU64::new(0));
    let writer = Arc::new(Mutex::new(writer));

    let (job_sender, job_receiver) = mpsc::channel::<AnalysisJob>();

    std::thread::scope(|scope| {
        let worker_analysis = Arc::clone(&analysis);
        let worker_completed = Arc::clone(&latest_completed);
        let worker_writer = Arc::clone(&writer);

        scope.spawn(move || {
            let mut assembler: Option<Assembler> = None;
            while let Ok(job) = job_receiver.recv() {
                // Drain any pending jobs to jump straight to the latest if multiple arrived
                let mut latest_job = job;
                while let Ok(newer_job) = job_receiver.try_recv() {
                    latest_job = newer_job;
                }

                let (new_analysis, new_assembler) =
                    rebuild_analysis(&latest_job.documents, &latest_job.opts, assembler.take());
                assembler = new_assembler;

                let completed = worker_completed.load(Ordering::SeqCst);
                if latest_job.generation >= completed {
                    worker_completed.store(latest_job.generation, Ordering::SeqCst);
                    if let Ok(mut lock) = worker_analysis.write() {
                        *lock = new_analysis;
                    }
                    if let Ok(analysis_lock) = worker_analysis.read() {
                        if let Ok(mut w) = worker_writer.lock() {
                            let _ = publish_semantic_diagnostics(
                                &mut **w,
                                &latest_job.uri,
                                &analysis_lock,
                            );
                            let _ = publish_target(&mut **w, &latest_job.uri, &latest_job.opts);
                        }
                    }
                }
            }
        });

        let mut exit_err: Result<(), String> = Ok(());

        loop {
            let message = match read_message(reader) {
                Ok(Some(msg)) => msg,
                Ok(None) => break,
                Err(e) => {
                    exit_err = Err(e.to_string());
                    break;
                }
            };
            let method = message.get("method").and_then(Value::as_str);
            let id = message.get("id").cloned();

            if method == Some("exit") {
                break;
            }

            let Some(method) = method else { continue };
            let response = match method {
                "initialize" => Some(success(
                    id,
                    json!({
                        "capabilities": {
                            "textDocumentSync": 1,
                            "hoverProvider": true,
                            "documentSymbolProvider": true,
                            "definitionProvider": true,
                            "referencesProvider": true,
                            "renameProvider": {"prepareProvider": true},
                            "documentFormattingProvider": true,
                            "documentRangeFormattingProvider": true
                        },
                        "serverInfo": {
                            "name": "gazm",
                            "version": env!("CARGO_PKG_VERSION"),
                            "cpu": format_cpu(fallback_opts.cpu)
                        }
                    }),
                )),
                "shutdown" => Some(success(id, Value::Null)),
                "textDocument/didOpen" => {
                    if let Some((uri, text)) = open_document(&message) {
                        documents.insert(uri.clone(), text);
                        let target_opts = selected_opts(&uri);
                        current_generation += 1;
                        if let Ok(mut w) = writer.lock() {
                            if let Err(e) =
                                publish_diagnostics(&mut **w, &uri, &documents[&uri], target_opts)
                            {
                                exit_err = Err(e);
                                break;
                            }
                        }
                        let _ = job_sender.send(AnalysisJob {
                            generation: current_generation,
                            uri: uri.clone(),
                            documents: documents.clone(),
                            opts: target_opts.clone(),
                        });
                    }
                    None
                }
                "textDocument/didChange" => {
                    if let Some((uri, text)) = changed_document(&message) {
                        documents.insert(uri.clone(), text);
                        let target_opts = selected_opts(&uri);
                        current_generation += 1;
                        if let Ok(mut w) = writer.lock() {
                            if let Err(e) =
                                publish_diagnostics(&mut **w, &uri, &documents[&uri], target_opts)
                            {
                                exit_err = Err(e);
                                break;
                            }
                        }
                        let _ = job_sender.send(AnalysisJob {
                            generation: current_generation,
                            uri: uri.clone(),
                            documents: documents.clone(),
                            opts: target_opts.clone(),
                        });
                    }
                    None
                }
                "textDocument/didSave" => {
                    if let Some(uri) = document_uri(&message) {
                        if let Some(text) = documents.get(&uri) {
                            let target_opts = selected_opts(&uri);
                            current_generation += 1;
                            if let Ok(mut w) = writer.lock() {
                                if let Err(e) =
                                    publish_diagnostics(&mut **w, &uri, text, target_opts)
                                {
                                    exit_err = Err(e);
                                    break;
                                }
                            }
                            let _ = job_sender.send(AnalysisJob {
                                generation: current_generation,
                                uri: uri.clone(),
                                documents: documents.clone(),
                                opts: target_opts.clone(),
                            });
                        }
                    }
                    None
                }
                "textDocument/didClose" => {
                    if let Some(uri) = document_uri(&message) {
                        documents.remove(&uri);
                        let target_opts = selected_opts(&uri);
                        current_generation += 1;
                        if let Ok(mut w) = writer.lock() {
                            let notification = json!({
                                "jsonrpc": "2.0",
                                "method": "textDocument/publishDiagnostics",
                                "params": {"uri": uri, "diagnostics": []}
                            });
                            if let Err(e) =
                                write_message(&mut **w, &notification).map_err(|e| e.to_string())
                            {
                                exit_err = Err(e);
                                break;
                            }
                        }
                        let _ = job_sender.send(AnalysisJob {
                            generation: current_generation,
                            uri: uri.clone(),
                            documents: documents.clone(),
                            opts: target_opts.clone(),
                        });
                    }
                    None
                }
                "textDocument/documentSymbol" => {
                    let result = document_uri(&message)
                        .and_then(|uri| {
                            documents
                                .get(&uri)
                                .map(|text| document_symbols(text, selected_opts(&uri)))
                        })
                        .unwrap_or_default();
                    Some(success(id, Value::Array(result)))
                }
                "textDocument/hover" => {
                    let analysis_guard = analysis.read().unwrap_or_else(|e| e.into_inner());
                    let result = hover_result(
                        &message,
                        &documents,
                        &analysis_guard,
                        selected_opts(&document_uri(&message).unwrap_or_default()),
                    );
                    Some(success(id, result))
                }
                "textDocument/definition" => {
                    let analysis_guard = analysis.read().unwrap_or_else(|e| e.into_inner());
                    let result = definition_result(&message, &documents, &analysis_guard);
                    Some(success(id, result))
                }
                "textDocument/references" => {
                    let analysis_guard = analysis.read().unwrap_or_else(|e| e.into_inner());
                    let result = references_result(&message, &documents, &analysis_guard);
                    Some(success(id, result))
                }
                "textDocument/rename" => {
                    let analysis_guard = analysis.read().unwrap_or_else(|e| e.into_inner());
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        rename_result(&message, &documents, &analysis_guard)
                    }))
                    .unwrap_or(Value::Null);
                    Some(success(id, result))
                }
                "textDocument/prepareRename" => {
                    let analysis_guard = analysis.read().unwrap_or_else(|e| e.into_inner());
                    let result = prepare_rename_result(&message, &documents, &analysis_guard);
                    Some(success(id, result))
                }
                "textDocument/formatting" => {
                    Some(success(id, formatting_result(&message, &documents)))
                }
                "textDocument/rangeFormatting" => {
                    Some(success(id, range_formatting_result(&message, &documents)))
                }
                "initialized" | "$/cancelRequest" => None,
                _ if id.is_some() => Some(error_response(id, -32601, "method not found")),
                _ => None,
            };

            if let Some(response) = response {
                if let Ok(mut w) = writer.lock() {
                    if let Err(e) = write_message(&mut **w, &response).map_err(|e| e.to_string()) {
                        exit_err = Err(e);
                        break;
                    }
                }
            }
        }

        drop(job_sender);
        exit_err
    })
}

fn rebuild_analysis(
    documents: &HashMap<String, String>,
    opts: &Opts,
    previous: Option<Assembler>,
) -> (ProjectAnalysis, Option<Assembler>) {
    catch_unwind(AssertUnwindSafe(|| {
        ProjectAnalysis::build(documents, opts, previous)
    }))
    .unwrap_or_else(|_| (ProjectAnalysis::default(), None))
}

fn publish_semantic_diagnostics(
    writer: &mut impl Write,
    uri: &str,
    analysis: &ProjectAnalysis,
) -> Result<(), String> {
    let mut diagnostics = analysis.diagnostics_for(&uri_to_path(uri));
    if let Some(mismatch) = analysis.first_reference_mismatch() {
        diagnostics.push(json!({
            "range": {
                "start": {"line": 0, "character": 0},
                "end": {"line": 0, "character": 1}
            },
            "severity": 1,
            "source": "gazm",
            "message": format!(
                "Reference {} differs at ${:04X}: generated ${:02X}, expected ${:02X}; later mismatches suppressed",
                mismatch.file.display(),
                mismatch.logical_addr,
                mismatch.generated,
                mismatch.expected
            ),
            "code": "reference-mismatch"
        }));
    }
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "diagnostics": diagnostics}
    });
    write_message(writer, &notification).map_err(|e| e.to_string())
}

fn document_uri(message: &Value) -> Option<String> {
    message
        .get("params")?
        .get("textDocument")?
        .get("uri")?
        .as_str()
        .map(str::to_owned)
}

fn open_document(message: &Value) -> Option<(String, String)> {
    let uri = document_uri(message)?;
    let text = message
        .get("params")?
        .get("textDocument")?
        .get("text")?
        .as_str()?
        .to_owned();
    Some((uri, text))
}

fn changed_document(message: &Value) -> Option<(String, String)> {
    let uri = document_uri(message)?;
    let text = message
        .get("params")?
        .get("contentChanges")?
        .as_array()?
        .last()?
        .get("text")?
        .as_str()?
        .to_owned();
    Some((uri, text))
}

fn publish_diagnostics(
    writer: &mut impl Write,
    uri: &str,
    text: &str,
    opts: &Opts,
) -> Result<(), String> {
    if text.trim().is_empty() {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {"uri": uri, "diagnostics": []}
        });
        return write_message(writer, &notification).map_err(|e| e.to_string());
    }

    let source = SourceFile::new(uri_to_path(uri), text, AsmSource::FromStr);
    let result = TokenizeRequest::for_single_source_file(source, opts).to_result();
    let diagnostics = result
        .errors
        .to_vec()
        .into_iter()
        .map(|error| {
            let (line, column) = error.position.line_col();
            let length = error.position.range().len().max(1);
            let raw_source_line = text.lines().nth(line).unwrap_or_default();
            let source_line = raw_source_line.trim();
            let kind = match &error.kind {
                FrontEndErrorKind::Unexpected => raw_source_line
                    .get(column..)
                    .and_then(|rest| rest.chars().next())
                    .map(|character| format!("Unexpected character `{character}`"))
                    .unwrap_or_else(|| error.kind.to_string()),
                _ => error.kind.to_string(),
            };
            let message = if source_line.is_empty() {
                format!("{} (line {}, column {})", kind, line + 1, column + 1)
            } else {
                format!(
                    "{} (line {}, column {})\n{}",
                    kind,
                    line + 1,
                    column + 1,
                    source_line
                )
            };
            json!({
                "range": {
                    "start": {"line": line, "character": column},
                    "end": {"line": line, "character": column + length}
                },
                "severity": if error.severity == Severity::Fatal { 1 } else { 2 },
                "source": "gazm",
                "message": message
            })
        })
        .collect::<Vec<_>>();
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "diagnostics": diagnostics}
    });
    write_message(writer, &notification).map_err(|e| e.to_string())
}

fn uri_to_path(uri: &str) -> PathBuf {
    uri.strip_prefix("file://")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(uri))
}

fn document_symbols(text: &str, opts: &Opts) -> Vec<Value> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let source = SourceFile::new("<lsp>", text, AsmSource::FromStr);
    let result = TokenizeRequest::for_single_source_file(source, opts).to_result();
    result
        .node
        .iter()
        .filter_map(|node| {
            let name = match &node.node.item {
                AstNodeKind::Label(definition)
                | AstNodeKind::LocalLabel(definition)
                | AstNodeKind::Assignment(definition)
                | AstNodeKind::AssignmentFromPc(definition)
                | AstNodeKind::LocalAssignment(definition)
                | AstNodeKind::LocalAssignmentFromPc(definition) => {
                    definition.get_text()?.to_owned()
                }
                _ => return None,
            };
            let position = node.node.ctx;
            let (line, column) = position.line_col();
            let length = position.range().len().max(name.len());
            Some(json!({
                "name": name,
                "kind": 12,
                "range": {
                    "start": {"line": line, "character": column},
                    "end": {"line": line, "character": column + length}
                },
                "selectionRange": {
                    "start": {"line": line, "character": column},
                    "end": {"line": line, "character": column + length}
                },
                "detail": "label"
            }))
        })
        .collect()
}

fn hover_result(
    message: &Value,
    documents: &HashMap<String, String>,
    analysis: &ProjectAnalysis,
    opts: &Opts,
) -> Value {
    let Some(uri) = document_uri(message) else {
        return Value::Null;
    };
    let Some(text) = documents.get(&uri) else {
        return Value::Null;
    };
    let Some(position) = message.get("params").and_then(|p| p.get("position")) else {
        return Value::Null;
    };
    let line = position.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
    let column = position
        .get("character")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let Some(source) = text.lines().nth(line) else {
        return Value::Null;
    };
    let path = uri_to_path(&uri);
    let word = analysis
        .symbol_at(&path, line, column)
        .map(|(name, _)| name)
        .or_else(|| {
            let w = word_at(source, column)?;
            let defined = document_symbols(text, opts)
                .iter()
                .any(|symbol| symbol["name"].as_str() == Some(w));
            defined.then_some(w)
        });
    let Some(word) = word else {
        return Value::Null;
    };
    json!({"contents": {"kind": "markdown", "value": format!("`{word}` — Gazm label")}})
}

fn definition_result(
    message: &Value,
    documents: &HashMap<String, String>,
    analysis: &ProjectAnalysis,
) -> Value {
    let Some(uri) = document_uri(message) else {
        return Value::Null;
    };
    let Some(text) = documents.get(&uri) else {
        return Value::Null;
    };
    let Some(position) = message.get("params").and_then(|p| p.get("position")) else {
        return Value::Null;
    };
    let line = position.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
    let column = position
        .get("character")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let Some(source) = text.lines().nth(line) else {
        return Value::Null;
    };
    let path = uri_to_path(&uri);
    let word = analysis
        .symbol_at(&path, line, column)
        .map(|(name, _)| name)
        .or_else(|| word_at(source, column));
    let Some(word) = word else {
        return Value::Null;
    };
    if let Some(location) = analysis.definition(word) {
        return Value::Array(vec![location]);
    }
    Value::Null
}

fn rename_result(
    message: &Value,
    documents: &HashMap<String, String>,
    analysis: &ProjectAnalysis,
) -> Value {
    let Some(uri) = document_uri(message) else {
        return Value::Null;
    };
    let Some(text) = documents.get(&uri) else {
        return Value::Null;
    };
    let Some(position) = message.get("params").and_then(|p| p.get("position")) else {
        return Value::Null;
    };
    let line = position.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
    let column = position
        .get("character")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let path = uri_to_path(&uri);
    let old_name = analysis
        .symbol_at(&path, line, column)
        .map(|(name, _)| name)
        .or_else(|| {
            text.lines()
                .nth(line)
                .and_then(|source| word_at(source, column))
        });
    let Some(old_name) = old_name else {
        return Value::Null;
    };
    let Some(new_name) = message
        .get("params")
        .and_then(|params| params.get("newName"))
        .and_then(Value::as_str)
    else {
        return Value::Null;
    };
    if !is_valid_symbol_name(new_name) || !analysis.has_occurrences(old_name) {
        return Value::Null;
    }
    analysis.rename(old_name, new_name)
}

fn prepare_rename_result(
    message: &Value,
    documents: &HashMap<String, String>,
    analysis: &ProjectAnalysis,
) -> Value {
    let Some(uri) = document_uri(message) else {
        return Value::Null;
    };
    let Some(text) = documents.get(&uri) else {
        return Value::Null;
    };
    let Some(position) = message.get("params").and_then(|p| p.get("position")) else {
        return Value::Null;
    };
    let line = position.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
    let column = position
        .get("character")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let (name, start, end) =
        if let Some((name, position)) = analysis.symbol_at(&uri_to_path(&uri), line, column) {
            (
                name.to_owned(),
                position.col(),
                position.col() + position.range().len(),
            )
        } else {
            let Some(source) = text.lines().nth(line) else {
                return Value::Null;
            };
            let Some(name) = word_at(source, column) else {
                return Value::Null;
            };
            let start = column.min(source.len());
            let mut start = start;
            while start > 0 && is_identifier_char(source.as_bytes()[start - 1] as char) {
                start -= 1;
            }
            let mut end = column.min(source.len());
            while end < source.len() && is_identifier_char(source.as_bytes()[end] as char) {
                end += 1;
            }
            (name.to_owned(), start, end)
        };
    json!({
        "range": {
            "start": {"line": line, "character": start},
            "end": {"line": line, "character": end}
        },
        "placeholder": name
    })
}

fn is_valid_symbol_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || matches!(first, '_' | '.' | '$' | '!'))
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '$' | '!')
        })
}

fn references_result(
    message: &Value,
    documents: &HashMap<String, String>,
    analysis: &ProjectAnalysis,
) -> Value {
    let Some(uri) = document_uri(message) else {
        return Value::Null;
    };
    let Some(text) = documents.get(&uri) else {
        return Value::Null;
    };
    let Some(position) = message.get("params").and_then(|p| p.get("position")) else {
        return Value::Null;
    };
    let line = position.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
    let column = position
        .get("character")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let word = text
        .lines()
        .nth(line)
        .and_then(|source| word_at(source, column))
        .or_else(|| {
            analysis
                .symbol_at(&uri_to_path(&uri), line, column)
                .map(|(name, _)| name)
        });
    let Some(word) = word else {
        return Value::Null;
    };
    // Return the definition as well as usages. This is more useful for Gazm's
    // project-wide symbols and avoids client-specific declaration filtering.
    Value::Array(analysis.references(word, true))
}

fn formatting_result(message: &Value, documents: &HashMap<String, String>) -> Value {
    let Some(uri) = document_uri(message) else {
        return Value::Null;
    };
    let Some(text) = documents.get(&uri) else {
        return Value::Null;
    };
    let formatted = format_text(text);
    if formatted == *text {
        return Value::Array(Vec::new());
    }

    let lines = text.split('\n').collect::<Vec<_>>();
    let end_line = lines.len().saturating_sub(1);
    let end_character = lines.last().map_or(0, |line| line.len());
    Value::Array(vec![json!({
        "range": {
            "start": {"line": 0, "character": 0},
            "end": {"line": end_line, "character": end_character}
        },
        "newText": formatted
    })])
}

fn range_formatting_result(message: &Value, documents: &HashMap<String, String>) -> Value {
    let Some(uri) = document_uri(message) else {
        return Value::Null;
    };
    let Some(text) = documents.get(&uri) else {
        return Value::Null;
    };
    let Some(range) = message.get("params").and_then(|params| params.get("range")) else {
        return Value::Null;
    };
    let start_line = range["start"]["line"].as_u64().unwrap_or(0) as usize;
    let requested_end = range["end"]["line"].as_u64().unwrap_or(start_line as u64) as usize;
    let lines = text.lines().collect::<Vec<_>>();
    if start_line >= lines.len() {
        return Value::Array(Vec::new());
    }
    let end_line = requested_end.min(lines.len() - 1);
    let old_text = lines[start_line..=end_line].join("\n");
    let new_text = format_text(&old_text);
    if old_text == new_text {
        return Value::Array(Vec::new());
    }
    Value::Array(vec![json!({
        "range": {
            "start": {"line": start_line, "character": 0},
            "end": {"line": end_line, "character": lines[end_line].len()}
        },
        "newText": new_text
    })])
}

fn word_at(source: &str, column: usize) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut start = column.min(bytes.len());
    while start > 0 && is_identifier_char(bytes[start - 1] as char) {
        start -= 1;
    }
    let mut end = column.min(bytes.len());
    while end < bytes.len() && is_identifier_char(bytes[end] as char) {
        end += 1;
    }
    (start < end).then(|| &source[start..end])
}

fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '$' | '!')
}

fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut content_length = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            if name.trim().eq_ignore_ascii_case("Content-Length") {
                content_length = Some(value.trim().parse::<usize>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid Content-Length: {e}"),
                    )
                })?);
            }
        }
    }

    let length = content_length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length"))?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("invalid JSON: {e}")))
}

fn write_message(writer: &mut impl Write, message: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(message)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

fn success(id: Option<Value>, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn error_response(id: Option<Value>, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reads_framed_json() {
        let body = br#"{"jsonrpc":"2.0","id":1,"method":"shutdown"}"#;
        let mut framed = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        framed.extend_from_slice(body);
        let message = read_message(&mut Cursor::new(framed)).unwrap().unwrap();
        assert_eq!(message["method"], "shutdown");
    }

    #[test]
    fn writes_framed_json() {
        let mut output = Vec::new();
        write_message(&mut output, &json!({"jsonrpc":"2.0","id":1,"result":null})).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.starts_with("Content-Length:"));
        let body = text.split_once("\r\n\r\n").unwrap().1;
        let message: Value = serde_json::from_str(body).unwrap();
        assert_eq!(message["jsonrpc"], "2.0");
        assert_eq!(message["id"], 1);
    }

    #[test]
    fn publishes_diagnostics_for_open_document() {
        let messages = [
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {}
            }),
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {"uri": "file:///tmp/test.gazm", "text": "nop\n"}
                }
            }),
            json!({"jsonrpc": "2.0", "method": "exit"}),
        ];
        let mut input = Vec::new();
        for message in messages {
            let body = serde_json::to_vec(&message).unwrap();
            write!(&mut input, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
            input.extend_from_slice(&body);
        }

        let mut output = Vec::new();
        run_server(
            &mut Cursor::new(input),
            &mut output,
            std::slice::from_ref(&Opts::default()),
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("textDocument/publishDiagnostics"));
    }

    #[test]
    fn finds_labels_for_symbols_and_hover() {
        let source = "start: nop\n      jmp start\n";
        let opts = Opts::default();
        let symbols = document_symbols(source, &opts);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0]["name"], "start");
        assert_eq!(symbols[0]["range"]["start"]["line"], 0);

        let mut documents = HashMap::new();
        documents.insert("file:///tmp/test.gazm".to_owned(), source.to_owned());
        let request = json!({
            "params": {
                "textDocument": {"uri": "file:///tmp/test.gazm"},
                "position": {"line": 1, "character": 11}
            }
        });
        assert!(
            hover_result(&request, &documents, &ProjectAnalysis::default(), &opts)["contents"]
                ["value"]
                .as_str()
                .unwrap()
                .contains("start")
        );
    }

    #[test]
    fn returns_full_document_formatting_edit() {
        let uri = "file:///tmp/test.gazm";
        let source = "start: nop\n    rts\n";
        let message = json!({"params": {"textDocument": {"uri": uri}}});
        let documents = HashMap::from([(uri.to_owned(), source.to_owned())]);
        let result = formatting_result(&message, &documents);
        assert_eq!(result[0]["range"]["start"]["line"], 0);
        assert_eq!(result[0]["range"]["end"]["line"], 2);
        assert_eq!(result[0]["newText"], format_text(source));
    }

    #[test]
    fn handles_definition_and_rapid_changes() {
        let messages = [
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {}
            }),
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {"uri": "file:///tmp/test.gazm", "text": "start:\n    nop\n    jmp start\n"}
                }
            }),
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {"uri": "file:///tmp/test.gazm"},
                    "contentChanges": [{"text": "start:\n    nop\n    jmp start\n"}]
                }
            }),
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "textDocument/definition",
                "params": {
                    "textDocument": {"uri": "file:///tmp/test.gazm"},
                    "position": {"line": 2, "character": 10}
                }
            }),
            json!({"jsonrpc": "2.0", "method": "exit"}),
        ];
        let mut input = Vec::new();
        for message in messages {
            let body = serde_json::to_vec(&message).unwrap();
            write!(&mut input, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
            input.extend_from_slice(&body);
        }

        let mut output = Vec::new();
        run_server(
            &mut Cursor::new(input),
            &mut output,
            std::slice::from_ref(&Opts::default()),
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("textDocument/publishDiagnostics"));
        assert!(output.contains("\"id\":1"));
        assert!(output.contains("\"id\":2"));
    }
}
