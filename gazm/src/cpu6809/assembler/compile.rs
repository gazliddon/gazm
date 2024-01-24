use crate::{
    assembler::{Assembler, AssemblerCpuTrait, BinaryError, Compiler, ScopeTracker, Sizer},
    error::GResult,
    frontend::{Item, Node, GazmParser, PResult},
    semantic::AstNodeId,
};

use grl_sources::ItemType;

use emu6809::isa;

use crate::cpu6809::{
    frontend::{AddrModeParseType, IndexParseType, MC6809},
    regutils::reg_pair_to_flags, regutils::registers_to_flags,
};


#[derive(Debug, Clone,Copy, PartialEq, Default)]
pub struct Assembler6809 {
    dp: Option<u8>,
}

impl From<Item<MC6809>> for grl_sources::ItemType {
    fn from(value: Item<MC6809>) -> Self {
        use grl_sources::ItemType::*;
        match value {
            Item::CpuSpecific(m) => match m {
                MC6809::Operand(..) => Other,
                MC6809::RegisterSet(..) => Other,
                MC6809::OperandIndexed(..) | MC6809::OpCode(..) => OpCode,
                MC6809::SetDp => Command,
                _ => todo!(),
            },
            _ => Other,
        }
    }
}

impl<'a> AssemblerCpuTrait for Assembler6809 {
    type NodeKind = MC6809;

    fn new() -> Self {
        Self{
            dp: None,
        }
    }

    fn parse_multi_opcode_vec(input: crate::frontend::TSpan) -> PResult<Vec<Node<Self::NodeKind>>> {
        GazmParser::parse_multi_opcode_vec(input)
    }

    fn parse_commands(input: crate::frontend::TSpan) -> PResult<Node<Self::NodeKind>> {
        GazmParser::parse_commands(input)
    }


    fn compile_node(
        compiler: &mut Compiler<Self>,
        asm: &mut Assembler<Self>,
        id: AstNodeId,
        node_kind: Self::NodeKind,
    ) -> GResult<()> {
        match node_kind {
            MC6809::OpCode(_, ins, amode) => {
                Self::compile_opcode(compiler, asm, id, &ins, amode)?;
            }
            _ => (),
        }
        Ok(())
    }

    fn size_node(
        sizer: &mut Sizer<Self>,
        asm: &mut Assembler<Self>,
        id: AstNodeId,
        node_kind: Self::NodeKind,
    ) -> GResult<()> {
        Self::size_node_internal(sizer,asm,id,node_kind)
    }
}
