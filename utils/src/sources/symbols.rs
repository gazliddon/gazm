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

    fn get_value(&self, name: &str) -> Result<i64, SymbolError> {
        let si = self.get_symbol_info(name)?;
        si.value.ok_or(SymbolError::NoValue)
    }

    fn symbol_exists_from_name(&self, name: &str) -> bool {
        self.get_symbol_info(name).is_ok()
    }
}

pub trait SymbolWriter {
    fn add_symbol_with_value(&mut self, name: &str, value: i64) -> Result<u64, SymbolError>;
    fn remove_symbol_name(&mut self, name: &str);
    fn add_symbol(&mut self, name: &str) -> Result<u64, SymbolError>;

    fn add_reference_symbol(&mut self, name: &str, val: i64);
}

////////////////////////////////////////////////////////////////////////////////
use serde::{Deserialize, Serialize};

// use crate::symbols::{ValueTraits, };
// use super::SymbolTree;
/// Holds information about a symbol
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SymbolInfo {
    /// Symbol Name
    pub name: String,
    // /// Unique Symbol Id
    // pub x_id: u64,
    /// Value, if any
    pub value: Option<i64>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SymbolError {
    AlreadyDefined(u64),
    Mismatch { expected: i64 },
    NotFound,
    NoValue,
}



