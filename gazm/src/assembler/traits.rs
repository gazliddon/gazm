use crate::error::GResult;
use crate::semantic::AstNodeId;
use super::Compiler;
use super::Assembler;
use super::Sizer;

pub trait AssemblerCpuTrait<K> {
    fn compile_node(
        &mut self,
        compiler: &mut Compiler,
        asm: &mut Assembler,
        id: AstNodeId,
        node_kind: K,
    ) -> GResult<()>;

    fn size_node(&mut self, sizer: &mut Sizer,_asm: &mut Assembler, _id: AstNodeId, _node_kind: K) -> GResult<()>;

}

