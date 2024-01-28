#[deny(unused_imports)]
use crate::{
    assembler::{self, AssemblerCpuTrait},
    error::GResult,
    frontend::{self, PResult, TSpan},
    semantic::AstNodeId,
};

use crate::cpu6809::{
    assembler::{compile_node, size_node_internal},
    frontend::{lex_identifier, parse_commands, parse_multi_opcode_vec, NodeKind6809},
};

pub type Node = frontend::Node<NodeKind6809>;
pub type Compiler<'a> = assembler::Compiler<'a, Asm6809>;
pub type Assembler = assembler::Assembler<Asm6809>;
pub type Sizer<'a> = assembler::Sizer<'a, Asm6809>;
pub type Item = crate::frontend::Item<NodeKind6809>;

#[inline]
pub fn from_item_tspan(item: Item, sp: TSpan) -> Node {
    frontend::from_item_tspan::<Asm6809>(item, sp)
}

#[inline]
pub fn from_item_kid_tspan(item: Item, node: Node, sp: TSpan) -> Node {
    frontend::from_item_kid_tspan::<Asm6809>(item, node, sp)
}

#[inline]
pub fn parse_expr(input: TSpan) -> PResult<Node> {
    frontend::parse_expr::<Asm6809>(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Asm6809 {
    dp: Option<u8>,
}

impl<'a> AssemblerCpuTrait for Asm6809 {
    type NodeKind = NodeKind6809;

    fn get_cpu_name() -> &'static str {
        "6809"
    }

    fn new() -> Self {
        Self { dp: None }
    }

    fn lex_identifier(_id: &str) -> crate::frontend::TokenKind {
        lex_identifier(_id)
    }

    fn parse_multi_opcode_vec(input: crate::frontend::TSpan) -> PResult<Vec<Node>> {
        parse_multi_opcode_vec(input)
    }

    fn parse_commands(input: crate::frontend::TSpan) -> PResult<Node> {
        parse_commands(input)
    }

    fn compile_node(
        compiler: &mut Compiler,
        asm: &mut Assembler,
        id: AstNodeId,
        node_kind: NodeKind6809,
    ) -> GResult<()> {
        compile_node(compiler, asm, id, node_kind)
    }

    fn size_node(
        sizer: &mut Sizer,
        asm: &mut Assembler,
        id: AstNodeId,
        node_kind: NodeKind6809,
    ) -> GResult<()> {
        size_node_internal(sizer, asm, id, node_kind)
    }
}