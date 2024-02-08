use std::env::current_exe;

use num_traits::cast;

use crate::{
    assembler::{ AssemblerCpuTrait,Assembler,Compiler },
    error::GResult,
    frontend::{PResult, TSpan, TokenKind, CpuSpecific, AstNodeKind, Node},
    semantic::{ AstNodeRef, AstNodeId },
};

use super::{
    frontend::{lex_identifier, parse_multi_opcode_vec, NodeKind6800},
};


#[derive(PartialEq, Debug, Default, Clone)]
pub struct Asm6800 {}

impl Asm6800 {
    pub fn new() -> Self {
        panic!()
    }
}


impl AssemblerCpuTrait for Asm6800 {
    // type NodeKind = NodeKind6800;

    fn get_cpu_name(&self) -> &'static str {
        "6800"
    }

    fn size_node(
        &self,
        _sizer: &mut crate::assembler::Sizer,
        _asm: &mut Assembler,
        _id: crate::semantic::AstNodeId,
        _node_kind: CpuSpecific,
        _current_scope_id: u64

    ) -> GResult<()> {
        panic!()
        // match node_kind {
        //     CpuSpecific::Cpu6800(node_kind) => size_node_internal(sizer, asm, id, node_kind),
        //     _ => panic!(),
        // }
    }

    fn compile_node(
        &self,
        asm: &mut Assembler,
        node: AstNodeRef,
        node_kind: CpuSpecific,
        current_scope_id: u64
    ) -> crate::error::GResult<()> {

        match node_kind {
            CpuSpecific::Cpu6800(node_kind) => asm.compile_node_6800( node_kind, node,current_scope_id),
            _ => panic!(),
        }
    }

    fn parse_multi_opcode_vec(&self,_input: crate::frontend::TSpan) -> PResult<Vec<Node>> {
        todo!()
        // parse_multi_opcode_vec(_input)
    }

    fn lex_identifier(&self,_id: &str) -> TokenKind {
        lex_identifier(_id)
    }
}
