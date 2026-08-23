//! Project-wide source analysis used by the language-server features.
//!
//! This module owns source discovery and assembly.  LSP requests should query
//! a [`ProjectAnalysis`] snapshot instead of reparsing the project for every
//! definition or reference request.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use grl_sources::{AsmSource, SourceFiles};
use serde_json::{json, Value};

use crate::{
    assembler::Assembler,
    error::{Diagnostic, DiagnosticSeverity, ErrorCollectorTrait, GazmErrorKind},
    frontend::AstNodeKind,
    opts::{BuildType, Opts},
    semantic::iter_refs_recursive,
};

#[derive(Debug, Default)]
pub(crate) struct ProjectAnalysis {
    definitions: HashMap<String, Value>,
    occurrences: HashMap<String, Vec<(PathBuf, grl_sources::Position)>>,
    diagnostics: Vec<(PathBuf, Value)>,
    first_reference_mismatch: Option<ReferenceMismatch>,
}

#[derive(Debug, Clone)]
pub(crate) struct ReferenceMismatch {
    pub file: PathBuf,
    pub logical_addr: usize,
    pub generated: usize,
    pub expected: usize,
}

impl ProjectAnalysis {
    /// Assemble the current open documents together with project sources.
    pub(crate) fn build(
        documents: &HashMap<String, String>,
        opts: &Opts,
        _previous: Option<Assembler>,
    ) -> (Self, Option<Assembler>) {
        let mut result = Self::default();
        if opts.project_file.as_os_str().is_empty() {
            return (result, _previous);
        }

        let base_dir = opts
            .project_file
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let mut sources = SourceFiles::with_base_dir(base_dir);
        for (uri, text) in project_documents(documents, opts) {
            sources.add_source_file(uri_to_path(&uri), &text);
        }

        let mut analysis_opts = opts.clone();
        analysis_opts.build_type = BuildType::Check;
        analysis_opts.no_async = true;
        analysis_opts.error_mismatches = false;
        // Rebuild from a fresh assembler when the set of open documents changes.
        // Reusing the previous source table can leave stale FileId entries behind
        // when Neovim opens the definition buffer after a goto-definition request.
        let mut assembler = Assembler::new_with_sources(analysis_opts.clone(), sources.clone());
        assembler.opts = analysis_opts;
        assembler.source_file_loader.sources = sources;
        assembler.collect_reference_mismatches();

        if let Err(error) = assembler.assemble() {
            for diagnostic in diagnostics_from_error(error) {
                let (line, character) = diagnostic.position.line_col();
                let length = diagnostic.position.range().len().max(1);
                let value = json!({
                    "range": {
                        "start": {"line": line, "character": character},
                        "end": {"line": line, "character": character + length}
                    },
                    "severity": match diagnostic.severity {
                        DiagnosticSeverity::Warning => 2,
                        DiagnosticSeverity::Note => 3,
                        _ => 1,
                    },
                    "source": "gazm",
                    "message": diagnostic.message,
                    "code": diagnostic.code,
                    "relatedInformation": diagnostic.help.as_ref().map(|help| vec![json!({"message": help})])
                });
                if let Some(file) = source_file_for(&assembler, diagnostic.position.src()) {
                    result.diagnostics.push((file, value));
                }
            }
        }

        if let Some(mismatch) = assembler.get_binary().reference_mismatches().first() {
            result.first_reference_mismatch = Some(ReferenceMismatch {
                file: mismatch.reference_file.clone(),
                logical_addr: mismatch.logical_addr,
                generated: mismatch.val,
                expected: mismatch.expected,
            });
        }

        if let Some(lookup) = assembler.asm_out.lookup.as_ref() {
            for (name, position) in lookup.references() {
                if let Some(file) = source_file_for(&assembler, position.src()) {
                    result
                        .occurrences
                        .entry(name)
                        .or_default()
                        .push((file, position));
                }
            }
            for (name, position) in lookup.definitions() {
                let Some(file) = source_file_for(&assembler, position.src()) else {
                    continue;
                };
                result
                    .occurrences
                    .entry(name.clone())
                    .or_default()
                    .push((file.clone(), position));
                let (line, character) = position.line_col();
                result.definitions.entry(name.clone()).or_insert_with(|| {
                    json!({
                        "uri": format!("file://{}", file.to_string_lossy()),
                        "range": {
                            "start": {"line": line, "character": character},
                            "end": {"line": line, "character": character + name.len()}
                        }
                    })
                });
            }
        } else if let Some(ast) = assembler.asm_out.ast.as_ref() {
            for node in iter_refs_recursive(ast.as_ref().root()) {
                let name = match &node.value().item {
                    AstNodeKind::Label(definition)
                    | AstNodeKind::LocalLabel(definition)
                    | AstNodeKind::Assignment(definition)
                    | AstNodeKind::AssignmentFromPc(definition)
                    | AstNodeKind::LocalAssignment(definition)
                    | AstNodeKind::LocalAssignmentFromPc(definition) => definition.get_text(),
                    _ => None,
                };
                let Some(name) = name else { continue };
                let Some(file) = source_file_for(&assembler, node.value().pos.src()) else {
                    continue;
                };
                let (line, character) = node.value().pos.line_col();
                result
                    .definitions
                    .entry(name.to_owned())
                    .or_insert_with(|| {
                        json!({
                            "uri": format!("file://{}", file.to_string_lossy()),
                            "range": {
                                "start": {"line": line, "character": character},
                                "end": {"line": line, "character": character + name.len()}
                            }
                        })
                    });
            }
        }

        (result, Some(assembler))
    }

    pub(crate) fn definition(&self, name: &str) -> Option<Value> {
        self.definitions.get(name).cloned()
    }

    pub(crate) fn diagnostics_for(&self, file: &Path) -> Vec<Value> {
        self.diagnostics
            .iter()
            .filter(|(diagnostic_file, _)| same_path(diagnostic_file, file))
            .map(|(_, diagnostic)| diagnostic.clone())
            .collect()
    }

    pub(crate) fn first_reference_mismatch(&self) -> Option<&ReferenceMismatch> {
        self.first_reference_mismatch.as_ref()
    }

    pub(crate) fn has_occurrences(&self, name: &str) -> bool {
        self.occurrences.contains_key(name)
    }

    /// Resolve a cursor position using the assembler's symbol occurrences.
    pub(crate) fn symbol_at(
        &self,
        file: &Path,
        line: usize,
        column: usize,
    ) -> Option<(&str, grl_sources::Position)> {
        self.occurrences.iter().find_map(|(name, occurrences)| {
            occurrences.iter().find_map(|(occurrence_file, position)| {
                let (start_line, start_column) = position.line_col();
                let end_column = start_column + position.range().len();
                (same_path(occurrence_file, file)
                    && line == start_line
                    && column >= start_column
                    && column <= end_column)
                    .then_some((name.as_str(), *position))
            })
        })
    }

    pub(crate) fn references(&self, name: &str, include_declaration: bool) -> Vec<Value> {
        self.occurrences
            .get(name)
            .into_iter()
            .flatten()
            .filter_map(|(file, position)| {
                let uri = format!("file://{}", file.to_string_lossy());
                let (line, character) = position.line_col();
                let length = position.range().len().max(1);
                let is_declaration = self.definitions.get(name).is_some_and(|definition| {
                    definition["uri"].as_str() == Some(uri.as_str())
                        && definition["range"]["start"]["line"] == line
                        && definition["range"]["start"]["character"] == character
                });
                if is_declaration && !include_declaration {
                    return None;
                }
                Some(json!({
                    "uri": uri,
                    "range": {
                        "start": {"line": line, "character": character},
                        "end": {"line": line, "character": character + length}
                    }
                }))
            })
            .collect()
    }

    pub(crate) fn rename(&self, old_name: &str, new_name: &str) -> Value {
        let mut changes: HashMap<String, Vec<Value>> = HashMap::new();
        for (file, position) in self.occurrences.get(old_name).into_iter().flatten() {
            let uri = format!("file://{}", file.to_string_lossy());
            let (line, character) = position.line_col();
            changes.entry(uri).or_default().push(json!({
                "range": {
                    "start": {"line": line, "character": character},
                    "end": {"line": line, "character": character + position.range().len()}
                },
                "newText": new_name
            }));
        }
        json!({"changes": changes})
    }
}

fn source_file_for(assembler: &Assembler, source: AsmSource) -> Option<PathBuf> {
    match source {
        AsmSource::FileId(id) => assembler
            .source_file_loader
            .sources
            .get_source_file_from_id(id)
            .ok()
            .map(|source| source.file.clone()),
        AsmSource::FromStr => None,
    }
}

fn diagnostics_from_error(error: GazmErrorKind) -> Vec<Diagnostic> {
    match error {
        GazmErrorKind::Diagnostic(diagnostic) => vec![diagnostic],
        GazmErrorKind::Diagnostics(bag) => bag.as_slice().to_vec(),
        GazmErrorKind::WithContext(_, inner) => diagnostics_from_error(*inner),
        _ => Vec::new(),
    }
}

fn project_documents(documents: &HashMap<String, String>, opts: &Opts) -> Vec<(String, String)> {
    let mut result = documents
        .iter()
        .map(|(uri, text)| (uri.clone(), text.clone()))
        .collect::<Vec<_>>();
    let Some(root) = opts
        .assemble_dir
        .clone()
        .or_else(|| opts.project_file.parent().map(PathBuf::from))
    else {
        return result;
    };
    collect_project_sources(&root, &mut result, documents);
    result
}

fn collect_project_sources(
    dir: &Path,
    result: &mut Vec<(String, String)>,
    open_documents: &HashMap<String, String>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_project_sources(&path, result, open_documents);
        } else if path.extension().is_some_and(|ext| ext == "gazm") {
            let uri = format!("file://{}", path.to_string_lossy());
            if !open_documents.contains_key(&uri) {
                if let Ok(text) = fs::read_to_string(&path) {
                    result.push((uri, text));
                }
            }
        }
    }
}

fn uri_to_path(uri: &str) -> PathBuf {
    uri.strip_prefix("file://")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(uri))
}

fn same_path(left: &Path, right: &Path) -> bool {
    left == right
        || left
            .canonicalize()
            .ok()
            .zip(right.canonicalize().ok())
            .is_some_and(|(left, right)| left == right)
}
