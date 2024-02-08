#[deny(unused_imports)]
use crate::{
    assembler::{Assembler, AssemblerCpuTrait,  Sizer},
    error::GResult,
    frontend::{self, PResult},
    semantic::{ AstNodeRef, AstNodeId },
};

use crate::{
    cpu6809::{
        assembler::{compile_node, size_node_internal},
        frontend::{ lex_identifier, },
    },
    frontend::CpuSpecific,
};

impl Assembler {
    pub fn compile_node_6809(
        &mut self,
        _node_kind: NodeKind6809,
        _node: AstNodeRef,
        _current_scope_id: u64,
    ) -> GResult<()> { 
        panic!()
}

    pub fn size_node_6809(
        &mut self,
        _sizer: &mut Sizer,
        _id: AstNodeId,
        _node_kind: NodeKind6809,
        _current_scope_id: u64,
    ) -> GResult<()> {
        panic!()
    }

}

use frontend::Node;

use super::frontend::NodeKind6809;
pub type NodeKind = frontend::AstNodeKind;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Asm6809 {
    dp: Option<u8>,
}

impl Asm6809 {
    pub fn new() -> Self {
        Self { dp: None }
    }
}

impl AssemblerCpuTrait for Asm6809 {
    // type NodeKind = NodeKind6809;

    fn get_cpu_name(&self) -> &'static str {
        "6809"
    }

    // fn new() -> Self {
    //     Self { dp: None }
    // }

    fn lex_identifier(&self, _id: &str) -> crate::frontend::TokenKind {
        lex_identifier(_id)
    }

    fn parse_multi_opcode_vec(&self, _input: crate::frontend::TSpan) -> PResult<Vec<Node>> {
        todo!()
        // parse_multi_opcode_vec(input)
    }

    fn parse_commands(&self, _input: crate::frontend::TSpan) -> PResult<Node> {
        todo!()
        // parse_commands(input)
    }

    fn compile_node(
        &self,
        asm: &mut Assembler,
        node: AstNodeRef,
        node_kind: CpuSpecific,
        current_scope_id: u64,
    ) -> GResult<()> {
        match node_kind {
            CpuSpecific::Cpu6809(node_kind) => compile_node(asm, node, node_kind, current_scope_id),
            _ => panic!(),
        }
    }

    fn size_node(
        &self,
        sizer: &mut Sizer,
        asm: &mut Assembler,
        id: AstNodeId,
        node_kind: CpuSpecific,
        _current_scope_id: u64
    ) -> GResult<()> {
        match node_kind {
            CpuSpecific::Cpu6809(node_kind) => size_node_internal(sizer, asm, id, node_kind),
            _ => panic!(),
        }
    }
}
