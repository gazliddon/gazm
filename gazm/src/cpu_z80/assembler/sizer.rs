#![forbid(unused_imports)]

use crate::cpu_z80::frontend::NodeKindZ80;

use crate::{
    assembler::{Assembler, Sizer},
    error::GResult,
    semantic::AstNodeId,
};

pub fn size_node_internal(
    sizer: &mut Sizer,
    _asm: &mut Assembler,
    _id: AstNodeId,
    node_kind: NodeKindZ80,
) -> GResult<()> {
    match &node_kind {
        NodeKindZ80::OpCode(ins, _) => {
            let instruction = crate::cpu_z80::assembler::ISA_DBASE
                .get_by_id(*ins)
                .unwrap();
            sizer.advance_pc(instruction.size);
        }
        NodeKindZ80::Illegal => panic!(),
    }
    Ok(())
}
