#![forbid(unused_imports)]

use crate::{
    error::GResult,
    frontend::{Node, PResult, TSpan, TokenKind, CpuSpecific},
    semantic::{ AstNodeId, AstNodeRef},
};

use super::{Assembler, Sizer};


pub trait AssemblerCpuTrait {

    fn get_cpu_name(&self) -> &'static str;

    fn err(&self, text: &str) -> GResult<()> {
        let err = format!("{text} :: {}", self.get_cpu_name());
        Err(crate::error::GazmErrorKind::NotImplemented(err))
    }

    fn compile_node(
        &self,
        _asm: &mut Assembler,
        _node: AstNodeRef,
        _node_kind: CpuSpecific,
        _current_scope_id: u64
    ) -> GResult<()> {
        self.err("Compile Node")
    }

    fn size_node(
        &self,
        _sizer: &mut Sizer,
        _asm: &mut Assembler,
        _id: AstNodeId,
        _node_kind: CpuSpecific,
        _current_scope_id: u64
    ) -> GResult<()> {
        self.err("Size Node")
    }

    fn parse_multi_opcode_vec(&self,_input: TSpan) -> PResult<Vec<Node>> {
        todo!()
        // err_nomatch(input)
    }

    fn parse_commands(&self,_input: TSpan) -> PResult<Node> {
        todo!()
        // err_nomatch(input)
    }

    fn lex_identifier(&self,_id: &str) -> TokenKind;
}
