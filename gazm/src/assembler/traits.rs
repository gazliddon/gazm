#![forbid(unused_imports)]

use crate::{
    error::GResult,
    frontend::{err_nomatch, Node, PResult, TSpan, TokenKind, CpuSpecific},
    semantic::AstNodeId,
};

use super::{Assembler, Compiler, Sizer};

use std::{default::Default, fmt::Debug};

pub trait AssemblerCpuTrait: Sized + Send + 'static + Debug + Clone + Default + PartialEq {
    type NodeKind: Debug + Clone + PartialEq + Send;

    fn new() -> Self;

    fn get_cpu_name() -> &'static str;

    fn err(text: &str) -> GResult<()> {
        let err = format!("{text} :: {}", Self::get_cpu_name());
        Err(crate::error::GazmErrorKind::NotImplemented(err))
    }

    fn compile_node(
        _compiler: &mut Compiler<Self>,
        _asm: &mut Assembler<Self>,
        _id: AstNodeId,
        _node_kind: CpuSpecific
    ) -> GResult<()> {
        Self::err("Compile Node")
    }

    fn size_node(
        _sizer: &mut Sizer<Self>,
        _asm: &mut Assembler<Self>,
        _id: AstNodeId,
        _node_kind: CpuSpecific
    ) -> GResult<()> {
        Self::err("Size Node")
    }

    fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node<Self::NodeKind>>> {
        err_nomatch(input)
    }

    fn parse_commands(input: TSpan) -> PResult<Node<Self::NodeKind>> {
        err_nomatch(input)
    }

    fn lex_identifier(_id: &str) -> TokenKind;
}
