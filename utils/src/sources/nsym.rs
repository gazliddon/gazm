
use crate::symbols::*;
use crate::sources::Position;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Undefined,
    Macro,
    Signed(i64),
    Unsigned(u64),
    Text(String),
    Double(f64),
}

pub type Symbols = Scopes<SymbolValue,usize>;
pub type SymbolId = ScopedSymbolId<usize>;

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolValue {
    value : Value,
    pos : Option<Position>,
}

impl ValueTraits for SymbolValue {
}

impl IdTraits for usize {
}

impl Default for SymbolValue {
    fn default() -> Self {
        Self {
            value: Value::Undefined,
            pos : None,
        }
    }
}




