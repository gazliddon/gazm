//! Per-target bundle: a `.map` + `.sym` pair for one CPU.

use crate::envelope::{decode_artifact, Artifact, ArtifactError, Magic, TargetInfo};
use crate::sourcemap::{SourceMap, SourceMapError};
use crate::symbols::{Symbols, SymbolsError};

/// Errors from `Target::load`.
#[derive(Debug)]
pub enum TargetError {
    Artifact(ArtifactError),
    SourceMap(SourceMapError),
    Symbols(SymbolsError),
    MismatchedTargets {
        map: Option<String>,
        sym: Option<String>,
    },
}

impl std::fmt::Display for TargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetError::Artifact(e) => write!(f, "artifact error: {e}"),
            TargetError::SourceMap(e) => write!(f, "source map error: {e}"),
            TargetError::Symbols(e) => write!(f, "symbols error: {e}"),
            TargetError::MismatchedTargets { map, sym } => write!(
                f,
                "map and sym describe different targets (map: {:?}, sym: {:?})",
                map, sym
            ),
        }
    }
}

impl std::error::Error for TargetError {}

impl From<ArtifactError> for TargetError {
    fn from(e: ArtifactError) -> Self {
        TargetError::Artifact(e)
    }
}
impl From<SourceMapError> for TargetError {
    fn from(e: SourceMapError) -> Self {
        TargetError::SourceMap(e)
    }
}
impl From<SymbolsError> for TargetError {
    fn from(e: SymbolsError) -> Self {
        TargetError::Symbols(e)
    }
}

/// One CPU's decoded metadata: identity header (when present), the
/// source map, and the symbol table.
pub struct Target {
    pub info: Option<TargetInfo>,
    pub source_map: SourceMap,
    pub symbols: Symbols,
}

impl Target {
    /// Decode a `.map`/`.sym` pair.  If both carry a `TargetInfo`
    /// header, their `target_name` and `cpu` must agree.
    pub fn from_artifacts<'a>(map: &Artifact<'a>, sym: &Artifact<'a>) -> Result<Self, TargetError> {
        let source_map = SourceMap::from_artifact(map)?;
        let symbols = Symbols::from_artifact(sym)?;

        if let (Some(m), Some(s)) = (&map.target_info, &sym.target_info) {
            if m.target_name != s.target_name || m.cpu != s.cpu {
                return Err(TargetError::MismatchedTargets {
                    map: Some(m.target_name.clone()),
                    sym: Some(s.target_name.clone()),
                });
            }
        }

        Ok(Target {
            info: map.target_info.clone().or_else(|| sym.target_info.clone()),
            source_map,
            symbols,
        })
    }

    /// Convenience: read both files from disk.
    pub fn load(map_bytes: &[u8], sym_bytes: &[u8]) -> Result<Self, TargetError> {
        let map = decode_artifact(map_bytes, Magic::SourceMap)?;
        let sym = decode_artifact(sym_bytes, Magic::Symbols)?;
        Self::from_artifacts(&map, &sym)
    }
}
