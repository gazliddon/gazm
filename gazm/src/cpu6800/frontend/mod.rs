mod lexer;
mod parse_opcode;

pub use lexer::*;
pub use parse_opcode::*;

use crate::cpu6800::Assembler6800;

use emu6800::cpu_core::{ AddrModeEnum, OpcodeData };
use crate::frontend::GazmParser;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum MC6800 {
    #[default]
    Illegal,
    OpCode(String,OpcodeData, AddrModeEnum),
    Operand,
    OperandIndexed,
}


pub type GParser = GazmParser<Assembler6800>;
pub type Node = crate::frontend::Node<MC6800>;
pub type Item  = crate::frontend::Item<MC6800>;


