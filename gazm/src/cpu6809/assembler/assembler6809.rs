#[deny(unused_imports)]
use crate::{
    assembler::{Assembler, AssemblerCpuTrait, Compiler, Sizer},
    cpu6809::frontend::lex_identifier,
    error::GResult,
    frontend::{GazmParser, Node, PResult},
    semantic::AstNodeId,
};

use crate::cpu6809::frontend::MC6809;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Assembler6809 {
    dp: Option<u8>,
}

impl<'a> AssemblerCpuTrait for Assembler6809 {
    type NodeKind = MC6809;

    fn new() -> Self {
        Self { dp: None }
    }

    fn lex_identifier(_id: &str) -> crate::frontend::TokenKind {
        lex_identifier(_id)
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
        Self::size_node_internal(sizer, asm, id, node_kind)
    }
}
