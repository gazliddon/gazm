#![forbid(unused_imports)]
use super::Assembler;

use crate::{
    astformat, debug_mess,
    error::GResult,
    info_mess, interesting_mess,
    messages::{info, status},
    opts::BinReference,
    status_err,
};

use grl_sources::SourceDatabase;
use grl_utils::{hash::get_hash, FileIo};

use anyhow::Context as AnyContext;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

// Version 3 reflects the compact source-mapping indexes and single-separator
// symbol syntax serialization; version 4 adds the optional `TargetInfo`
// header block (flag bit 0). Files without the header stay version 3 so
// consumers that predate the header keep loading them (contract §3/§4).
const ARTIFACT_VERSION: u16 = 3;
/// v4: header present, bincode (positional). Frozen for old-file compat.
const ARTIFACT_VERSION_WITH_HEADER: u16 = 4;
/// v5: header present, rmp-serde named map (evolvable — contract §4).
const ARTIFACT_VERSION_NAMED_HEADER: u16 = 5;
const FLAG_HAS_TARGET_INFO: u16 = 0x0001;

/// Per-build target identity embedded at v4. Field order and types must
/// match `gazm-metadata`'s `TargetInfo` exactly — bincode 1.x is
/// layout-sensitive and the reader deserializes this block. Append-only:
/// new fields go at the end with `#[serde(default)]` (contract §4).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TargetInfo {
    target_name: String,
    cpu: crate::cpukind::CpuKind,
    mem_size: usize,
    exec_addr: Option<usize>,
    bin_references: Vec<BinReference>,
    checksums: Vec<RomChecksum>,
    sections: Vec<Section>,
    tool_version: String,
    #[serde(default)]
    struct_sizes: Vec<StructSize>,
}

/// A struct's total size in bytes, e.g. `Proc -> 15`. Mirror of
/// `gazm-metadata`'s `StructSize`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StructSize {
    name: String,
    size: usize,
}

/// Mirror of `gazm-metadata`'s `RomChecksum` (same field order).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RomChecksum {
    name: String,
    addr: usize,
    size: usize,
    sha1: String,
}

/// Mirror of `gazm-metadata`'s `Section` (same field order). Note the
/// `AccessType` variant order here matches the reader's
/// (`Read, Write, ReadWrite`), which differs from the assembler's own
/// `AccessType` — that is why this mirror exists.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Section {
    name: String,
    logical_range: std::ops::Range<usize>,
    physical_range: std::ops::Range<usize>,
    access: AccessType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AccessType {
    Read,
    Write,
    ReadWrite,
}

fn encode_artifact<T: Serialize>(
    magic: &[u8; 4],
    target_info: Option<&TargetInfo>,
    value: &T,
) -> GResult<Vec<u8>> {
    let payload = bincode::serialize(value).context("Unable to serialize binary artifact")?;

    // The header block is length-prefixed and inserted between the fixed
    // envelope and the payload; version and flags advertise it (contract
    // §4). Since v5 the header is a rmp-serde *named map*, so fields can
    // be added freely: readers default missing fields and ignore unknown
    // ones. v4 (bincode, positional) is frozen for old-file compat.
    let mut body = Vec::with_capacity(payload.len());
    let (version, flags) = if let Some(info) = target_info {
        let header =
            rmp_serde::to_vec_named(info).context("Unable to serialize TargetInfo header")?;
        body.extend_from_slice(&(header.len() as u64).to_le_bytes());
        body.extend_from_slice(&header);
        (ARTIFACT_VERSION_NAMED_HEADER, FLAG_HAS_TARGET_INFO)
    } else {
        (ARTIFACT_VERSION, 0)
    };
    body.extend_from_slice(&payload);

    let payload_len = u64::try_from(body.len()).context("Binary artifact is too large")?;
    let mut output = Vec::with_capacity(16 + body.len());
    output.extend_from_slice(magic);
    output.extend_from_slice(&version.to_le_bytes());
    output.extend_from_slice(&flags.to_le_bytes());
    output.extend_from_slice(&payload_len.to_le_bytes());
    output.extend_from_slice(&body);
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

    /// Serialize the independent metadata outputs together, then write them
    /// in parallel. Binary artifacts carry a versioned envelope so consumers
    /// can reject formats they do not understand.
    ///
    /// Writing is driven by the `metadata` switch (contract §5):
    /// `metadata = true` writes one file per target — `<target>.meta` —
    /// holding the source-map and symbols v5 envelopes concatenated back
    /// to back, each carrying the `TargetInfo` header. The old two-file
    /// `.map`/`.sym` layout is deprecated. Absent/false writes nothing.
    fn write_metadata_outputs(&self) -> GResult<()> {
        if !self.opts.metadata {
            return Ok(());
        }

        let (target, bundle_path) = self.metadata_paths()?;

        // The header is part of the metadata bundle.
        let target_info = Some(TargetInfo {
            target_name: target,
            cpu: self.opts.cpu,
            mem_size: self.opts.mem_size,
            exec_addr: self.asm_out.exec_addr,
            bin_references: self.opts.bin_references.clone(),
            checksums: self
                .opts
                .checksums
                .iter()
                .map(|(name, c)| RomChecksum {
                    name: name.clone(),
                    addr: c.addr,
                    size: c.size,
                    sha1: c.sha1.clone(),
                })
                .collect(),
            sections: self
                .asm_out
                .sections
                .iter()
                .map(|s| Section {
                    name: s.name.clone(),
                    logical_range: s.logical_range.clone(),
                    physical_range: s.physical_range.clone(),
                    access: match s.access_type {
                        crate::assembler::AccessType::ReadWrite => AccessType::ReadWrite,
                        crate::assembler::AccessType::ReadOnly => AccessType::Read,
                    },
                })
                .collect(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            struct_sizes: self
                .asm_out
                .struct_sizes
                .iter()
                .filter_map(|(scope_id, size)| {
                    let name = self
                        .get_symbols()
                        .get_scope_info_from_id(*scope_id)
                        .map(|info| info.name.clone());
                    name.map(|name| StructSize { name, size: *size })
                })
                .collect(),
        });

        // Both envelopes are built in parallel, then concatenated into
        // the single bundle file.
        let (source, symbols) = rayon::join(
            || {
                let mut database: SourceDatabase = self.into();
                database.file_name = bundle_path.clone();
                encode_artifact(b"GZMP", target_info.as_ref(), &database)
            },
            || encode_artifact(b"GZSY", target_info.as_ref(), self.get_symbols()),
        );
        let source = source?;
        let symbols = symbols?;

        let mut data = Vec::with_capacity(source.len() + symbols.len());
        data.extend_from_slice(&source);
        data.extend_from_slice(&symbols);

        if let Some(parent) = bundle_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Unable to create output directory {parent:?}"))?;
        }
        fs::write(&bundle_path, &data)
            .with_context(|| format!("Unable to write {bundle_path:?}"))?;
        interesting_mess!("Written metadata bundle: {}", bundle_path.to_string_lossy());
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

    /// Derived `<target>.map` / `<target>.sym` output paths (contract §5).
    /// The target name is the configured `[[targets]] name`, falling back
    /// to the project file stem. Shared by the metadata and deps writers so
    /// the derivation lives in one place.
    fn metadata_paths(&self) -> GResult<(String, std::path::PathBuf)> {
        let target = self.opts.target_name.clone().unwrap_or_else(|| {
            self.opts
                .project_file
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "gazm".to_string())
        });
        Ok((target.clone(), self.output_path(format!("{target}.meta"))?))
    }

    pub fn write_deps_file(&mut self) -> GResult<()> {
        if let Some(deps) = &self.opts.deps_file {
            if self.opts.metadata {
                let (_, bundle_file) = self.metadata_paths()?;
                let deps = self.output_path(deps)?;
                let sf = self.get_source_file_loader();
                let read = join_paths(sf.get_files_read().iter(), " \\\n");
                let written = join_paths(sf.get_files_written().iter(), " \\\n");
                let deps_line_2 = format!("{written} : {:?}", bundle_file);
                let deps_line = format!("{deps_line_2}\n{:?} : {read}", bundle_file);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opts::Opts;

    use gazm_metadata::envelope::{AccessType as ReaderAccess, CpuKind as ReaderCpu};
    use gazm_metadata::{decode_artifact, Magic};

    fn sample_target_info() -> TargetInfo {
        TargetInfo {
            target_name: "stargate".to_string(),
            cpu: crate::cpukind::CpuKind::Cpu6809,
            mem_size: 94208,
            exec_addr: Some(0xE000),
            bin_references: vec![BinReference {
                file: "orig/roms/01".into(),
                addr: 0x0000,
            }],
            checksums: vec![RomChecksum {
                name: "rom_1".into(),
                addr: 0x0000,
                size: 0x1000,
                sha1: "f003a5a9319c4eb8991fa2aae3f10c72d6b8e81a".into(),
            }],
            sections: vec![Section {
                name: "lo_rom".into(),
                logical_range: 0x0000..0x9000,
                physical_range: 0x0000..0x9000,
                access: AccessType::Read,
            }],
            tool_version: "test".into(),
            struct_sizes: vec![StructSize {
                name: "Proc".into(),
                size: 15,
            }],
        }
    }

    /// No header: version 3, flags 0, `target_info = None` — the layout
    /// v3 consumers already read.
    #[test]
    fn artifact_without_header_round_trips_as_v3() {
        let payload: Vec<u32> = vec![1, 2, 3];
        let bytes = encode_artifact(b"GZMP", None, &payload).unwrap();

        let art = decode_artifact(&bytes, Magic::SourceMap).unwrap();
        assert_eq!(art.version, ARTIFACT_VERSION);
        assert_eq!(art.flags, 0);
        assert!(art.target_info.is_none());

        let decoded: Vec<u32> = bincode::deserialize(art.payload).unwrap();
        assert_eq!(decoded, payload);
    }

    /// With header: version 5 (named map), flag bit 0, and every
    /// `TargetInfo` field decodes through the real reader with identical
    /// values.
    #[test]
    fn artifact_with_header_round_trips_through_reader() {
        let info = sample_target_info();
        let payload: Vec<u32> = vec![7, 8];
        let bytes = encode_artifact(b"GZSY", Some(&info), &payload).unwrap();

        let art = decode_artifact(&bytes, Magic::Symbols).unwrap();
        assert_eq!(art.version, ARTIFACT_VERSION_NAMED_HEADER);
        assert_eq!(art.flags, FLAG_HAS_TARGET_INFO);

        let got = art.target_info.expect("header present");
        assert_eq!(got.target_name, "stargate");
        assert_eq!(got.cpu, ReaderCpu::Cpu6809);
        assert_eq!(got.mem_size, 94208);
        assert_eq!(got.exec_addr, Some(0xE000));
        assert_eq!(got.bin_references.len(), 1);
        assert_eq!(got.bin_references[0].file.to_string_lossy(), "orig/roms/01");
        assert_eq!(got.bin_references[0].addr, 0x0000);
        assert_eq!(got.checksums.len(), 1);
        assert_eq!(got.checksums[0].name, "rom_1");
        assert_eq!(
            got.checksums[0].sha1,
            "f003a5a9319c4eb8991fa2aae3f10c72d6b8e81a"
        );
        assert_eq!(got.sections.len(), 1);
        assert_eq!(got.sections[0].name, "lo_rom");
        assert_eq!(got.sections[0].logical_range, 0x0000..0x9000);
        assert_eq!(got.sections[0].access, ReaderAccess::Read);
        assert_eq!(got.tool_version, "test");

        let decoded: Vec<u32> = bincode::deserialize(art.payload).unwrap();
        assert_eq!(decoded, payload);
    }

    /// End to end: `metadata = true` writes `<target>.map` + `<target>.sym`
    /// with the header, the in-asm sections are persisted, and the reader's
    /// `Target::load` accepts the pair.
    #[test]
    fn metadata_bundle_written_by_assembler_loads_in_reader() {
        let src = r#"
            section rom_01, start = $1000, size = $1000
            nop
            section dp_ram, start = $9800, size = $100
            rmb 2
        "#;
        let dir = std::env::temp_dir();
        let path = dir.join("gazm_metadata_bundle_test.gazm");
        std::fs::write(&path, src).unwrap();

        let opts = Opts {
            project_file: path.clone(),
            assemble_dir: Some(dir.clone()),
            metadata: true,
            target_name: Some("mytest".to_string()),
            ..Default::default()
        };
        let mut asm = Assembler::new(opts);
        let res = asm.assemble();
        assert!(res.is_ok(), "Assembly failed: {:?}", res.err());
        asm.write_outputs().unwrap();

        let bundle = dir.join("mytest.meta");
        let target = gazm_metadata::Target::load_file(&bundle).unwrap();
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&bundle);

        let info = target.info.expect("v5 header present");
        assert_eq!(info.target_name, "mytest");
        assert_eq!(info.cpu, ReaderCpu::Cpu6809);

        let rom = info
            .sections
            .iter()
            .find(|s| s.name == "rom_01")
            .expect("rom_01 persisted");
        assert_eq!(rom.logical_range.start, 0x1000);
        assert!(rom.logical_range.end > 0x1000);
        assert_eq!(rom.access, ReaderAccess::ReadWrite);

        // The single `nop` must show up as an instruction boundary.
        assert!(!target.source_map.boundaries().is_empty());
    }

    /// metadata absent + no explicit paths -> nothing written.
    #[test]
    fn metadata_absent_writes_nothing() {
        let dir = std::env::temp_dir();
        let path = dir.join("gazm_metadata_absent_test.gazm");
        std::fs::write(&path, "nop\n").unwrap();

        let opts = Opts {
            project_file: path.clone(),
            assemble_dir: Some(dir.clone()),
            target_name: Some("quiet".to_string()),
            ..Default::default()
        };
        let mut asm = Assembler::new(opts);
        let res = asm.assemble();
        assert!(res.is_ok(), "Assembly failed: {:?}", res.err());
        asm.write_outputs().unwrap();
        let _ = std::fs::remove_file(&path);

        assert!(!dir.join("quiet.map").exists());
        assert!(!dir.join("quiet.sym").exists());
    }
}
