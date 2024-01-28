use crate::assembler::AssemblerCpuTrait;
use crate::frontend::{PResult, TSpan, TokenKind};

use super::frontend::{lex_identifier, parse_commands, parse_multi_opcode_vec, MC6800};

pub type Node = crate::frontend::Node<MC6800>;
pub type Item = crate::frontend::Item<MC6800>;
pub type Compiler<'a> = crate::assembler::Compiler<'a, Asm6800>;
pub type Assembler = crate::assembler::Assembler<Asm6800>;
pub type Sizer<'a> = crate::assembler::Sizer<'a, Asm6800>;

#[derive(PartialEq, Debug, Default, Clone)]
pub struct Asm6800 {}

#[inline]
pub fn from_item_tspan<I>(item: I, sp: TSpan) -> Node
where
    I: Into<Item>,
{
    crate::frontend::from_item_tspan::<Asm6800>(item.into(), sp)
}

#[inline]
pub fn from_item_kid_tspan<I>(item: Item, node: Node, sp: TSpan) -> Node 
where
    I: Into<Item>,
{
    crate::frontend::from_item_kid_tspan::<Asm6800>(item.into(), node, sp)
}

#[inline]
pub fn parse_expr(input: TSpan) -> PResult<Node> {
    crate::frontend::parse_expr::<Asm6800>(input)
}

impl AssemblerCpuTrait for Asm6800 {
    type NodeKind = MC6800;

    fn new() -> Self {
        Self {}
    }

    fn get_cpu_name() -> &'static str {
        "6800"
    }

    fn parse_multi_opcode_vec(_input: crate::frontend::TSpan) -> PResult<Vec<Node>> {
        parse_multi_opcode_vec(_input)
    }

    fn lex_identifier(_id: &str) -> TokenKind {
        lex_identifier(_id)
    }
}
