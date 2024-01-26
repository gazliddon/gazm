use crate::assembler::{ AssemblerCpuTrait,Compiler, Assembler, Sizer, };
use crate::cpu6800::frontend::lex_identifier;
use crate::frontend::{ PResult, Node, TokenKind };

use super::frontend::MC6800;

#[derive(PartialEq, Debug, Default, Clone)]
pub struct Assembler6800 {}

impl AssemblerCpuTrait for Assembler6800 {

    type NodeKind = MC6800;

    fn new() -> Self {
        Self {}
    }

    fn compile_node(
        _compiler: &mut Compiler<Self>,
        _asm: &mut Assembler<Self>,
        _id: crate::semantic::AstNodeId,
        _node_kind: Self::NodeKind,
    ) -> crate::error::GResult<()> {
        todo!()
    }

    fn size_node(
        _sizer: &mut Sizer<Self>,
        _asm: &mut Assembler<Self>,
        _id: crate::semantic::AstNodeId,
        _node_kind: Self::NodeKind,
    ) -> crate::error::GResult<()> {
        todo!()
    }

    fn parse_commands(_input: crate::frontend::TSpan) -> PResult<Node<Self::NodeKind>> {
        todo!()
    }

    fn parse_multi_opcode_vec(
        _input: crate::frontend::TSpan,
    ) -> crate::frontend::PResult<Vec<Node<Self::NodeKind>>> {
        todo!()
    }

    fn lex_identifier(_id: &str) -> TokenKind {
        lex_identifier(_id)
    }
}
