use emu6800::cpu_core::{AddrModeEnum, OpcodeData, RegEnum, DBASE};

use crate::cpu6800::Asm6800;
use crate::frontend::{ GazmParser, CpuSpecific, AstNodeKind, Node };


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
    OpCode(Box<String>, Box<OpcodeData>),
    Operand(AddrModeParseType),
}

impl NodeKind6800 {
    pub fn opcode<T: Into<String>>(name : T, opcode_data: &OpcodeData) -> Self {
        NodeKind6800::OpCode(Box::new(name.into()), Box::new(opcode_data.clone()))
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

