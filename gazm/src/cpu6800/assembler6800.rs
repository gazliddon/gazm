use crate::{
    assembler::{Assembler, AssemblerCpuTrait, Sizer},
    error::GResult,
    frontend::{self, CpuSpecific, PResult},
    semantic::{AstNodeId, AstNodeRef},
};

use super::frontend::lex_identifier;

#[derive(PartialEq, Debug, Default, Clone)]
pub struct Asm6800 {}

impl Asm6800 {
    pub fn new() -> Self {
        Self {}
    }
}

impl AssemblerCpuTrait for Asm6800 {
    fn get_cpu_name(&self) -> &'static str {
        "6800"
    }

    fn lex_identifier(&self, id: &str) -> crate::frontend::TokenKind {
        lex_identifier(id)
    }

    fn parse_multi_opcode_vec<'a>(
        &self,
        input: crate::frontend::TSpan<'a>,
    ) -> PResult<'a, Vec<frontend::Node>> {
        crate::cpu6800::frontend::parse_multi_opcode_vec(input)
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
            CpuSpecific::Cpu6800(node_kind) => {
                asm.compile_node_6800(node_kind, node, current_scope_id)
            }
            _ => panic!(),
        }
    }

    fn size_node(
        &self,
        sizer: &mut Sizer,
        asm: &mut Assembler,
        id: AstNodeId,
        node_kind: CpuSpecific,
        current_scope_id: u64,
    ) -> GResult<()> {
        match node_kind {
            CpuSpecific::Cpu6800(node_kind) => {
                asm.size_node_6800(sizer, id, node_kind, current_scope_id)
            }
            _ => panic!(),
        }
    }
}
