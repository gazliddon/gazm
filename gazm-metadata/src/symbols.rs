//! Symbol queries over a decoded `GZSY` artifact.

use crate::envelope::{Artifact, Magic};
use grl_symbols::SymbolTree;

/// A resolved symbol: name, scope-qualified name, and value (address).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub scoped_name: String,
    pub value: i64,
}

/// Errors from `Symbols::from_artifact`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolsError {
    WrongMagic,
    BadPayload(String),
}

impl std::fmt::Display for SymbolsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolsError::WrongMagic => write!(f, "not a GZSY symbols artifact"),
            SymbolsError::BadPayload(e) => write!(f, "bad symbols payload: {e}"),
        }
    }
}

impl std::error::Error for SymbolsError {}

/// Decoded symbol table with an address-sorted index for lookups.
pub struct Symbols {
    tree: SymbolTree<u64, u64, i64>,
    by_addr: Vec<Symbol>,
}

impl Symbols {
    /// Decode a `GZSY` artifact.  The payload is bincode-serialized
    /// `SymbolTree` (via its `Seriablizable` mirror); deserializing
    /// rebuilds the internal tree.
    pub fn from_artifact(artifact: &Artifact<'_>) -> Result<Self, SymbolsError> {
        if artifact.magic != Magic::Symbols {
            return Err(SymbolsError::WrongMagic);
        }
        let tree: SymbolTree<u64, u64, i64> = bincode::deserialize(artifact.payload)
            .map_err(|e| SymbolsError::BadPayload(e.to_string()))?;
        let mut by_addr: Vec<Symbol> = tree
            .symbols()
            .values()
            .filter_map(|info| {
                info.value.map(|value| Symbol {
                    name: info.name().to_string(),
                    scoped_name: info.scoped_name().to_string(),
                    value,
                })
            })
            .collect();
        by_addr.sort_by_key(|s| s.value);
        Ok(Self { tree, by_addr })
    }

    /// The symbol with the largest value at or below `addr`
    /// (binary search over the sorted index).
    pub fn symbol_at(&self, addr: i64) -> Option<&Symbol> {
        let idx = self.by_addr.partition_point(|s| s.value <= addr);
        if idx == 0 {
            None
        } else {
            Some(&self.by_addr[idx - 1])
        }
    }

    /// The symbol whose value equals `addr` exactly.
    pub fn exact_symbol(&self, addr: i64) -> Option<&Symbol> {
        self.by_addr
            .binary_search_by_key(&addr, |s| s.value)
            .ok()
            .map(|idx| &self.by_addr[idx])
    }

    /// First symbol with value >= `name`'s address... resolves a name
    /// to its address via the tree (scope-aware).
    pub fn address_of(&self, name: &str) -> Option<i64> {
        let reader = self.tree.get_root_reader();
        reader
            .get_symbol_info(name)
            .ok()
            .and_then(|info| info.value)
    }

    /// All symbols with values in `[start, end]`.
    pub fn symbols_in_range(&self, start: i64, end: i64) -> Vec<&Symbol> {
        let lo = self.by_addr.partition_point(|s| s.value < start);
        let hi = self.by_addr.partition_point(|s| s.value <= end);
        self.by_addr[lo..hi].iter().collect()
    }

    /// All symbols, sorted by value.
    pub fn all(&self) -> &[Symbol] {
        &self.by_addr
    }
}
