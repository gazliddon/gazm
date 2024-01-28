mod lexer;
mod error;
mod commands;
mod parse_opcode;
mod register;

pub use lexer::*;
pub use parse_opcode::*;
pub use error::*;
pub use commands::*;
pub use register::*;

use crate::cpu6800::Asm6800;

use emu6800::cpu_core::{ AddrModeEnum, OpcodeData };
use crate::frontend::GazmParser;

use super::AddrModeParseType;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum MC6800 {
    #[default]
    Illegal,
    OpCode(String,OpcodeData, AddrModeParseType),
    Operand(AddrModeParseType),
}

pub type GParser = GazmParser<Asm6800>;
pub type Node = crate::frontend::Node<MC6800>;
pub type Item  = crate::frontend::Item<MC6800>;

impl From<MC6800> for Item {
    fn from(value: MC6800) -> Self {
        Item::CpuSpecific(value)
    }
}

impl From<AddrModeParseType> for Item {
    fn from(value: AddrModeParseType) -> Self {
        Item::CpuSpecific(MC6800::Operand(value))
    }
}


