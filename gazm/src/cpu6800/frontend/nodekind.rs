use emu6800::cpu_core::{AddrModeEnum, OpcodeId, RegEnum, DBASE};

use crate::cpu6800::Asm6800;
use crate::frontend::{AstNodeKind, CpuSpecific, GazmParser, Node};

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
    OpCode(OpcodeId),
    Operand(AddrModeParseType),
}

impl NodeKind6800 {
    pub fn opcode<I: Into<OpcodeId>>(opcode_id: I) -> Self {
        NodeKind6800::OpCode(opcode_id.into())
    }
}

impl From<NodeKind6800> for AstNodeKind {
    fn from(value: NodeKind6800) -> Self {
        AstNodeKind::TargetSpecific(CpuSpecific::Cpu6800(value))
    }
}

impl From<AddrModeParseType> for AstNodeKind {
    fn from(value: AddrModeParseType) -> Self {
        NodeKind6800::Operand(value).into()
    }
}
