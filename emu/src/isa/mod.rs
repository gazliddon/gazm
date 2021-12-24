use serde_derive::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Deserialize)]
pub enum AddrModeEnum {
    Indexed,
    Direct,
    Extended,
    Relative,
    Relative16,
    Inherent,
    Immediate8,
    Immediate16,
    Stack,
}
impl fmt::Display for AddrModeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

mod isa_reader;
pub use isa_reader::*;
