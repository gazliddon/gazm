use emu6800::cpu::ISA_DBASE;
use emu6800::cpu_core::{AddrModeEnum, DBASE};

use crate::debug_mess;
use crate::{
    assembler::{Assembler, Sizer},
    error::GResult,
    semantic::AstNodeId,
};

use crate::cpu6800::frontend::NodeKind6800;

impl Assembler {
    pub fn size_node_6800(
        &mut self,
        sizer: &mut Sizer,
        id: AstNodeId,
        node_kind: NodeKind6800,
        current_scope_id: u64,
    ) -> GResult<()> {
        size_node_internal(sizer, self, id, node_kind, current_scope_id)
    }
}

fn size_node_internal(
    sizer: &mut Sizer,
    asm: &mut Assembler,
    id: AstNodeId,
    node_kind: NodeKind6800,
    current_scope_id: u64,
) -> GResult<()> {
    let node = sizer.get_node(id);

    use NodeKind6800::*;

    match &node_kind {
        Illegal => todo!(),

        OpCode(text, ins) => {
            // get the size of this instruction
            let mut size = ins.size;

            let ins_info = ISA_DBASE
                .get_instruction_info_from_opcode(ins.opcode)
                .unwrap();

            if ins_info.addr_mode == AddrModeEnum::Extended {
                // Is this extend addressing and we support direct?
                // If see evaluate the operand and see if the result is
                // in the first page
                // If it is we can do direct addressing

                let new_ins = DBASE
                    .get_instruction_info_from_opcode(ins.opcode)
                    .and_then(|i_type| i_type.instruction.get_opcode_data(AddrModeEnum::Direct));

                if let Some(new_ins) = new_ins {
                    if let Ok((value, _)) = asm.eval_first_arg(node, current_scope_id) {
                        if ((value as u64) >> 8) & 0xff == 0 {
                            let src = asm.get_source_info(&node.value().pos);

                            if let Ok(src) = src {
                                debug_mess!("Xformed from Extended to Direct :  {}", src.line_str);
                            }

                            let new_ins = new_ins.clone();
                            size = new_ins.size;
                            let new_item = OpCode(text.clone(), new_ins.into());

                            asm.add_fixup(id, new_item, current_scope_id);
                        }
                    }
                }
            }

            sizer.advance_pc(size)
        }

        Operand(_) => todo!(),
    }
    Ok(())
}
