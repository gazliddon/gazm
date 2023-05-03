use emu::utils::sources::Position;
use emu::utils::symbols;

use symbols::{
    Scopes,
    ValueTraits,
};

use super::Value;

pub type Symbols = Scopes<SymbolValue, usize>;
#[allow(dead_code)]
pub type SymbolId = symbols::SymbolId<usize>;

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolValue {
    value: Value,
    pos: Option<Position>,
}

#[allow(dead_code)]
impl SymbolValue {
    pub fn new(value: Value, pos: Option<Position>) -> Self {
        Self { value, pos }
    }

    pub fn new_text(text: &str, pos: Option<Position>) -> Self {
        Self::new(Value::Text(text.to_string()), pos)
    }

    pub fn new_double(num: f64, pos: Option<Position>) -> Self {
        Self::new(Value::Float(num), pos)
    }
    pub fn new_signed(num: i64, pos: Option<Position>) -> Self {
        Self::new(Value::Signed(num), pos)
    }
    pub fn new_unsigned(num: u64, pos: Option<Position>) -> Self {
        Self::new(Value::Unsigned(num), pos)
    }
}

impl ValueTraits for SymbolValue {}

impl Default for SymbolValue {
    fn default() -> Self {
        Self {
            value: Value::Null,
            pos: None,
        }
    }
}

#[allow(unused_imports)]
#[allow(dead_code)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};


    fn test_sym() {
    }

    // #[test]
    fn create_scopes() {
        let mut syms = Symbols::new();
        let x = syms.create_scope("root", "test").unwrap();
        println!("{x:?}");
        panic!()
    }

    fn find_a_scope() {}

    fn write_symbols_to_scope() {}

    fn resolve_a_symbol_from_a_scope() {}

    fn find_a_symbol_in_a_scope() {}

    // Need to
    // Create scopes
    // Write a symbol to scope
    // Resolve a symbol from within a scope
    //    Walk up the tree
    // Find a symbol in a scope
}

