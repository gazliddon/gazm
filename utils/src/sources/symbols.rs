////////////////////////////////////////////////////////////////////////////////
// Traits

pub trait Value {
    fn value(&self) -> Option<i64>;
}

// A symbol table has 3 things
// An Id
// An info struct

pub trait SymbolQuery {
    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError>;

    fn get_symbol_info_from_id(&self, id: SymbolScopeId) -> Result<&SymbolInfo, SymbolError>;

    fn get_value(&self, name: &str) -> Result<i64, SymbolError> {
        let si = self.get_symbol_info(name)?;
        si.value.ok_or(SymbolError::NoValue)
    }

    fn get_value_from_id(&self, id: SymbolScopeId) -> Result<i64, SymbolError> {
        let si = self.get_symbol_info_from_id(id)?;
        si.value.ok_or(SymbolError::NoValue)
    }

    fn symbol_exists_from_name(&self, name: &str) -> bool {
        self.get_symbol_info(name).is_ok()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct SymbolScopeId {
    pub scope_id : u64,
    pub symbol_id: u64,
}

pub trait SymbolWriter {
    fn add_symbol_with_value(&mut self, name: &str, value: i64) -> Result<SymbolScopeId, SymbolError>;
    fn remove_symbol_name(&mut self, name: &str);
    fn add_symbol(&mut self, name: &str) -> Result<SymbolScopeId, SymbolError>;
    fn add_reference_symbol(&mut self, name: &str, val: i64);
    fn set_symbol(&mut self, symbol_id : SymbolScopeId, val : i64) -> Result<(), SymbolError>;
}

////////////////////////////////////////////////////////////////////////////////
use serde::{Deserialize, Serialize};

// use crate::symbols::{ValueTraits, };
// use super::SymbolTree;
/// Holds information about a symbol
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SymbolInfo {
    /// Symbol Name
    name: String,
    // /// Unique Symbol Id
    // pub x_id: u64,
    /// Value, if any
    pub value: Option<i64>,

    pub symbol_id : SymbolScopeId,
}

impl SymbolInfo {
    pub fn new(name: &str, value : Option<i64>, symbol_id : SymbolScopeId) -> Self {
        Self {
            name: name.to_string(), value, symbol_id 
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SymbolError {
    InvalidScope,
    AlreadyDefined(( u64, Option<i64> )),
    Mismatch { expected: i64 },
    NotFound,
    NoValue,
}



