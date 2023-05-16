////////////////////////////////////////////////////////////////////////////////
// Traits

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct SymbolScopeId {
    pub scope_id: u64,
    pub symbol_id: u64,
}

////////////////////////////////////////////////////////////////////////////////
use serde::{Deserialize, Serialize};

// use crate::symbols::{ValueTraits, };
// use super::SymbolTree;
/// Holds information about a symbol
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SymbolInfo {
    name: String,
    scoped_name: String,
    pub value: Option<i64>,
    pub symbol_id: SymbolScopeId,
}

impl SymbolInfo {
    pub fn new(name: &str, value: Option<i64>, symbol_id: SymbolScopeId, fqn: &str) -> Self {
        Self {
            name: name.to_string(),
            value,
            symbol_id,
            scoped_name : format!("{fqn}::{name}")
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn scoped_name(&self) -> &str {
        &self.scoped_name
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SymbolError {
    InvalidScope,
    AlreadyDefined(SymbolScopeId),
    Mismatch { expected: i64 },
    NotFound,
    NoValue,
    InvalidId,
}
