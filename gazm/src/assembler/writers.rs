#![forbid(unused_imports)]
use super::Assembler;

use crate::{
    astformat, debug_mess,
    error::GResult,
    info_mess, interesting_mess,
    messages::{info, status},
    status_err,
};

use grl_sources::SourceDatabase;
use grl_utils::{hash::get_hash, FileIo};

use anyhow::Context as AnyContext;
use rayon::prelude::*;
use serde::Serialize;
use std::{fs, path::Path};

// Version 3 reflects the compact source-mapping indexes and single-separator
// symbol syntax serialization.
const ARTIFACT_VERSION: u16 = 3;

fn encode_artifact<T: Serialize>(magic: &[u8; 4], value: &T) -> GResult<Vec<u8>> {
    let payload = bincode::serialize(value).context("Unable to serialize binary artifact")?;
    let payload_len = u64::try_from(payload.len()).context("Binary artifact is too large")?;
    let mut output = Vec::with_capacity(16 + payload.len());
    output.extend_from_slice(magic);
    output.extend_from_slice(&ARTIFACT_VERSION.to_le_bytes());
    output.extend_from_slice(&0u16.to_le_bytes()); // reserved flags
    output.extend_from_slice(&payload_len.to_le_bytes());
    output.extend_from_slice(&payload);
    Ok(output)
}

fn join_paths<P: AsRef<Path>, I: Iterator<Item = P>>(i: I, sep: &str) -> String {
    let z: Vec<String> = i.map(|s| s.as_ref().to_string_lossy().into()).collect();
    z.join(sep)
}

impl Assembler {
    fn output_path<P: AsRef<Path>>(&self, path: P) -> GResult<std::path::PathBuf> {
        let path = self.expand_path_to_deprecate(path)?;
        Ok(if path.is_absolute() {
            path
        } else {
            self.cwd.join(path)
        })
    }

    /// Write any outputs that need writing
    pub fn write_outputs(&mut self) -> GResult<()> {
        status("Writing files", |_| {
            self.write_bin_chunks()?;
            self.checksum_report();
            self.write_metadata_outputs()?;
            self.write_deps_file()?;
            self.write_ast_file()?;
            Ok(())
        })
    }

    fn write_bin_chunks(&mut self) -> GResult<()> {
        for bin_to_write in &self.asm_out.bin_to_write_chunks {
            if let Some(parent) = bin_to_write.bin_desc.file.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Unable to create output directory {parent:?}"))?;
            }
        }
        info("Writing binary chunks", |_| {
            let writes = self
                .asm_out
                .bin_to_write_chunks
                .iter()
                .map(|bin_to_write| {
                    (
                        bin_to_write.data.clone(),
                        bin_to_write.bin_desc.file.clone(),
                        bin_to_write.bin_desc.addr.clone(),
                    )
                })
                .collect::<Vec<_>>();

            let results = writes
                .par_iter()
                .map(|(data, file, _range)| {
                    fs::write(file, data)
                        .with_context(|| format!("Unable to write binary file {file:?}"))
                })
                .collect::<Vec<_>>();

            for ((_, file, range), result) in writes.iter().zip(results) {
                result?;
                self.source_file_loader.add_to_files_written(file.clone());
                debug_mess!(
                    "Written binary: {:?} ${:x} ${:x}",
                    file,
                    range.start,
                    range.len()
                );
            }
            Ok::<(), anyhow::Error>(())
        })?;

        Ok(())
    }

    fn write_file<P: AsRef<Path>>(&mut self, p: P, txt: &str) -> GResult<String> {
        let full_file_name = self.output_path(p)?;
        if let Some(parent) = full_file_name.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Unable to create output directory {parent:?}"))?;
        }
        fs::write(&full_file_name, txt)
            .with_context(|| format!("Unable to write {:?}", full_file_name))?;
        Ok(full_file_name.to_string_lossy().into_owned())
    }

    /// Serialize the independent metadata outputs together, then write them
    /// in parallel. Binary artifacts carry a versioned header so consumers can
    /// reject formats they do not understand.
    fn write_metadata_outputs(&self) -> GResult<()> {
        let source_path = self
            .opts
            .source_mapping
            .as_ref()
            .map(|path| self.output_path(path))
            .transpose()?;
        let symbols_path = self
            .opts
            .syms_file
            .as_ref()
            .map(|path| self.output_path(path))
            .transpose()?;

        let (source, symbols) = rayon::join(
            || {
                source_path.map(|path| {
                    let mut database: SourceDatabase = self.into();
                    database.file_name = path.clone();
                    let data = if self.opts.json_output {
                        let text = if self.opts.pretty_json {
                            serde_json::to_string_pretty(&database)
                        } else {
                            serde_json::to_string(&database)
                        }
                        .context("Unable to serialize source mappings")?;
                        text.into_bytes()
                    } else {
                        encode_artifact(b"GZMP", &database)?
                    };
                    Ok::<_, anyhow::Error>((path, data, "source mappings"))
                })
            },
            || {
                symbols_path.map(|path| {
                    let data = if self.opts.json_output {
                        let text = if self.opts.pretty_json {
                            serde_json::to_string_pretty(self.get_symbols())
                        } else {
                            serde_json::to_string(self.get_symbols())
                        }
                        .context("Unable to serialize symbols")?;
                        text.into_bytes()
                    } else {
                        encode_artifact(b"GZSY", self.get_symbols())?
                    };
                    Ok::<_, anyhow::Error>((path, data, "symbols"))
                })
            },
        );

        let mut outputs = Vec::with_capacity(2);
        if let Some(output) = source {
            outputs.push(output?);
        }
        if let Some(output) = symbols {
            outputs.push(output?);
        }

        let results = outputs
            .par_iter()
            .map(|(path, data, _)| {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Unable to create output directory {parent:?}"))?;
                }
                fs::write(path, data).with_context(|| format!("Unable to write {path:?}"))
            })
            .collect::<Vec<_>>();

        for ((path, _, kind), result) in outputs.iter().zip(results) {
            result?;
            interesting_mess!("Written {kind}: {}", path.to_string_lossy());
        }
        Ok(())
    }

    pub fn write_ast_file(&mut self) -> GResult<()> {
        if let Some(ast_file) = &self.opts.ast_file {
            let ast_file = self.output_path(ast_file)?;
            if let Some(parent) = ast_file.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Unable to create output directory {parent:?}"))?;
            }

            interesting_mess!("Writing ast: {}", ast_file.to_string_lossy());

            if let Some(ast) = &self.asm_out.ast {
                let x = astformat::as_string(ast.as_ref().root());
                fs::write(&ast_file, x).with_context(|| {
                    format!("Unable to write list file {}", ast_file.to_string_lossy())
                })?;
            } else {
                status_err!("No AST file to write");
            }
        }
        Ok(())
    }

    pub fn write_deps_file(&mut self) -> GResult<()> {
        if let Some(deps) = &self.opts.deps_file {
            if let Some(sym_file) = &self.opts.source_mapping {
                let sym_file = self.output_path(sym_file)?;
                let deps = self.output_path(deps)?;
                let sf = self.get_source_file_loader();
                let read = join_paths(sf.get_files_read().iter(), " \\\n");
                let written = join_paths(sf.get_files_written().iter(), " \\\n");
                let deps_line_2 = format!("{written} : {:?}", sym_file);
                let deps_line = format!("{deps_line_2}\n{:?} : {read}", sym_file);

                interesting_mess!("Writing deps file: {deps:?}");

                if let Some(parent) = deps.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Unable to create output directory {parent:?}"))?;
                }

                std::fs::write(&deps, deps_line)
                    .with_context(|| format!("Unable to write {deps:?}"))?;
            }
        }

        Ok(())
    }

    pub fn write_sym_file(&mut self) -> GResult<()> {
        if let Some(syms_file) = &self.opts.syms_file {
            let syms_file = self.output_path(syms_file)?;
            let json_text = if self.opts.pretty_json {
                serde_json::to_string_pretty(self.get_symbols())
            } else {
                serde_json::to_string(self.get_symbols())
            }
            .context("Unable to serialize symbols")?;
            let file_name = self.write_file(syms_file, &json_text)?;
            interesting_mess!("Writen symbols file: {}", file_name);
        }

        Ok(())
    }

    fn write_source_mapping(&mut self) -> GResult<()> {
        if let Some(sym_file) = &self.opts.source_mapping {
            let sym_file = self.output_path(sym_file)?;
            if let Some(parent) = sym_file.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Unable to create output directory {parent:?}"))?;
            }
            info_mess!("Writing source mappings {}", sym_file.to_string_lossy());
            let sd: SourceDatabase = (&*self).into();
            let mut sd = sd;
            sd.file_name = sym_file.clone();
            let json = if self.opts.pretty_json {
                serde_json::to_string_pretty(&sd)
            } else {
                serde_json::to_string(&sd)
            }
            .context("Unable to serialize source mappings")?;
            fs::write(&sym_file, json).with_context(|| format!("Unable to write {sym_file:?}"))?;
        }

        Ok(())
    }

    fn checksum_report(&self) {
        if !self.opts.checksums.is_empty() {
            let mut errors = vec![];

            for (name, csum) in &self.opts.checksums {
                let data = self
                    .get_binary()
                    .get_bytes(csum.addr, csum.size)
                    .expect("Binary error");
                let this_hash = get_hash(data);
                let expected_hash = csum.sha1.to_lowercase();

                if this_hash != expected_hash {
                    let hash = format!("{name} : {this_hash} != {expected_hash}");
                    errors.push(hash);
                }
            }

            if errors.is_empty() {
                info_mess!("✅: {} Checksums correct", self.opts.checksums.len())
            } else {
                crate::messages::error_message(format_args!("❌ : Mismatched Checksums"));
                for name in errors {
                    status_err!("{name} : ❌");
                }
            }
        }
    }
}
