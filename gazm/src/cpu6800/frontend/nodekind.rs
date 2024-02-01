use emu6800::cpu_core::{AddrModeEnum, OpcodeData, RegEnum, DBASE};

use crate::cpu6800::Asm6800;
use crate::frontend::GazmParser;

pub type GParser = GazmParser<Asm6800>;
pub type Node = crate::frontend::Node<NodeKind6800>;
pub type AstNodeKind = crate::frontend::AstNodeKind<NodeKind6800>;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AddrModeParseType {
    Indexed,
    Direct,
    Extended,
    Relative,
    Inherent,
    Immediate,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum NodeKind6800 {
    #[default]
    Illegal,
    OpCode(String, OpcodeData),
    Operand(AddrModeParseType),
}

impl From<NodeKind6800> for AstNodeKind {
    fn from(value: NodeKind6800) -> Self {
        AstNodeKind::CpuSpecific(value)
    }
}

impl From<AddrModeParseType> for AstNodeKind {
    fn from(value: AddrModeParseType) -> Self {
        AstNodeKind::CpuSpecific(NodeKind6800::Operand(value))
    }
}
