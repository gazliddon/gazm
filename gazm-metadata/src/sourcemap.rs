//! Source-map queries over a decoded `GZMP` artifact.

use crate::envelope::{Artifact, Magic};
use grl_sources::{ItemType, SourceDatabase, SourceMapping};

/// A source position resolved from an address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file_id: u64,
    pub file_name: String,
    pub line: usize,
}

/// One instruction's address range and its source position, lifted from
/// the `OpCode` mappings (contract §3/§7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionBoundary {
    pub addr: usize,
    pub len: usize,
    pub file_id: u64,
    pub line: usize,
}

/// Errors from `SourceMap::from_artifact`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceMapError {
    WrongMagic,
    BadPayload(String),
}

impl std::fmt::Display for SourceMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceMapError::WrongMagic => write!(f, "not a GZMP source-map artifact"),
            SourceMapError::BadPayload(e) => write!(f, "bad source-map payload: {e}"),
        }
    }
}

impl std::error::Error for SourceMapError {}

/// Decoded source map: the `SourceDatabase` plus a prebuilt, sorted
/// instruction-boundary index.
pub struct SourceMap {
    db: SourceDatabase,
    boundaries: Vec<InstructionBoundary>,
}

impl SourceMap {
    /// Decode a `GZMP` artifact.  The payload is bincode-serialized
    /// `SourceDatabase`; `rebuild_indexes()` is required after
    /// deserializing (the serde impl skips runtime caches).
    pub fn from_artifact(artifact: &Artifact<'_>) -> Result<Self, SourceMapError> {
        if artifact.magic != Magic::SourceMap {
            return Err(SourceMapError::WrongMagic);
        }
        let mut db: SourceDatabase = bincode::deserialize(artifact.payload)
            .map_err(|e| SourceMapError::BadPayload(e.to_string()))?;
        db.rebuild_indexes();
        let boundaries = Self::build_boundaries(&db);
        Ok(Self { db, boundaries })
    }

    /// Instruction starts sorted by logical address, built from the
    /// `OpCode` mappings.  Note: runtime-generated code (e.g. Stargate's
    /// self-modifying RAM handlers) has no mappings, so this is a hint
    /// layer for the disassembler, not a complete code map.
    fn build_boundaries(db: &SourceDatabase) -> Vec<InstructionBoundary> {
        let mut out = Vec::new();
        for m in db.mappings().mappings.iter() {
            if m.item_type == ItemType::OpCode && !m.mem_range.is_empty() {
                out.push(InstructionBoundary {
                    addr: m.mem_range.start,
                    len: m.mem_range.len(),
                    file_id: m.file_id,
                    line: m.line,
                });
            }
        }
        out.sort_by_key(|b| b.addr);
        out
    }

    pub fn boundaries(&self) -> &[InstructionBoundary] {
        &self.boundaries
    }

    /// The instruction boundary whose start is at or before `addr`
    /// (binary search over the sorted index).
    pub fn boundary_at(&self, addr: usize) -> Option<&InstructionBoundary> {
        let idx = self.boundaries.partition_point(|b| b.addr <= addr);
        if idx == 0 {
            None
        } else {
            Some(&self.boundaries[idx - 1])
        }
    }

    /// Source location for a *logical* address (banked as executed).
    pub fn location_at(&self, addr: usize) -> Option<SourceLocation> {
        self.resolve(addr, true)
    }

    /// Source location for a *physical* address (ROM file offset).
    pub fn location_at_physical(&self, addr: usize) -> Option<SourceLocation> {
        self.resolve(addr, false)
    }

    fn resolve(&self, addr: usize, logical: bool) -> Option<SourceLocation> {
        let line = if logical {
            self.db.get_source_info_from_address(addr)
        } else {
            self.db.get_source_info_from_physical_address(addr)
        }?;
        Some(SourceLocation {
            file_id: line.file_id,
            file_name: line.file.to_string_lossy().into_owned(),
            line: line.line_number,
        })
    }

    /// The line text for a location, loading the source file on demand.
    pub fn source_text(&self, loc: &SourceLocation) -> Option<String> {
        Some(
            self.db
                .get_source_line_from_file(&loc.file_name, loc.line)?
                .text,
        )
    }

    /// The complete text of a source file, loaded on demand from the
    /// path recorded in the map.
    pub fn source_file_text(&self, file_id: u64) -> Option<String> {
        self.db.get_source_file(file_id)?.get_entire_source()
    }

    pub fn file_name(&self) -> &std::path::Path {
        &self.db.file_name
    }

    pub fn exec_addr(&self) -> Option<usize> {
        self.db.exec_addr
    }
}

/// Cheap accessors used by the debugger's memory view.
impl SourceMap {
    pub fn mappings(&self) -> &SourceMapping {
        self.db.mappings()
    }
}
