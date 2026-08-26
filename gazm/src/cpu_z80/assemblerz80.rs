use crate::{
    assembler::{Assembler, AssemblerCpuTrait, Sizer},
    error::GResult,
    frontend::{self, CpuSpecific, PResult},
    semantic::{AstNodeId, AstNodeRef},
};

use crate::cpu_z80::{
    assembler::{compile_node, size_node_internal},
    frontend::lex_identifier,
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AsmZ80 {}

impl AsmZ80 {
    pub fn new() -> Self {
        Self {}
    }
}

impl AssemblerCpuTrait for AsmZ80 {
    fn get_cpu_name(&self) -> &'static str {
        "Z80"
    }

    fn lex_identifier(&self, id: &str) -> crate::frontend::TokenKind {
        lex_identifier(id)
    }

    fn parse_multi_opcode_vec<'a>(
        &self,
        input: crate::frontend::TSpan<'a>,
    ) -> PResult<'a, Vec<frontend::Node>> {
        crate::cpu_z80::frontend::parse_multi_opcode_vec(input)
    }

    fn parse_commands<'a>(
        &self,
        _input: crate::frontend::TSpan<'a>,
    ) -> PResult<'a, frontend::Node> {
        todo!()
    }

    fn compile_node(
        &self,
        asm: &mut Assembler,
        node: AstNodeRef,
        node_kind: CpuSpecific,
        current_scope_id: u64,
    ) -> GResult<()> {
        match node_kind {
            CpuSpecific::CpuZ80(node_kind) => compile_node(asm, node, node_kind, current_scope_id),
            _ => panic!(),
        }
    }

    fn size_node(
        &self,
        sizer: &mut Sizer,
        asm: &mut Assembler,
        id: AstNodeId,
        node_kind: CpuSpecific,
        _current_scope_id: u64,
    ) -> GResult<()> {
        match node_kind {
            CpuSpecific::CpuZ80(node_kind) => size_node_internal(sizer, asm, id, node_kind),
            _ => panic!(),
        }
    }
}

pub type NodeKind = frontend::AstNodeKind;
