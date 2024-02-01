pub type GParser = GazmParser<Asm6800>;
pub type Node = crate::frontend::Node<NodeKind6800>;
pub type Item  = crate::frontend::Item<NodeKind6800>;

use crate::cpu6800::Asm6800;
use emu6800::cpu_core::{ AddrModeEnum, OpcodeData };
use crate::frontend::GazmParser;

use emu6800::cpu_core::DBASE;
use emu6800::cpu_core::RegEnum;

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
    OpCode(String,OpcodeData),
    Operand(AddrModeParseType),
}

impl From<NodeKind6800> for Item {
    fn from(value: NodeKind6800) -> Self {
        Item::CpuSpecific(value)
    }
}

impl From<AddrModeParseType> for Item {
    fn from(value: AddrModeParseType) -> Self {
        Item::CpuSpecific(NodeKind6800::Operand(value))
    }
}
