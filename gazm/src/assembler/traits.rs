
use crate::error::GResult;
use crate::frontend::{ TSpan, PResult, TokenKind };
use crate::semantic::AstNodeId;
use super::Compiler;
use super::Assembler;
use super::Sizer;
use crate::frontend::Node;


pub trait AssemblerCpuTrait : Sized  + Send + 'static + std::fmt::Debug + Clone + std::default::Default + PartialEq{
    type NodeKind: std::fmt::Debug + Clone + PartialEq + Send;

    fn new() -> Self;

    fn compile_node(
        compiler: &mut Compiler<Self>,
        asm: &mut Assembler<Self>,
        id: AstNodeId,
        node_kind: Self::NodeKind,
    ) -> GResult<()>;

    fn size_node(sizer: &mut Sizer<Self>,_asm: &mut Assembler<Self>, _id: AstNodeId, _node_kind: Self::NodeKind) -> GResult<()>;

    fn parse_multi_opcode_vec(_input: TSpan) -> PResult<Vec<Node<Self::NodeKind>>> ;

    fn parse_commands(_input: TSpan) -> PResult<Node<Self::NodeKind>> ;

    fn lex_identifier(_id: &str) -> TokenKind;
}

