use emu6800::cpu::ISA_DBASE;
use emu6800::cpu_core::{AddrModeEnum, DBASE};

use crate::debug_mess;
use crate::{error::GResult, semantic::AstNodeId};

use crate::cpu6800::{frontend::MC6800, Assembler, Item, Sizer};

pub fn size_node_internal(
    sizer: &mut Sizer,
    asm: &mut Assembler,
    id: AstNodeId,
    node_kind: MC6800,
) -> GResult<()> {
    let current_scope_id = sizer.scopes.scope();
    let node = sizer.get_node(id);

    use MC6800::*;

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
                            let new_item = Item::CpuSpecific(OpCode(text.clone(), new_ins));

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
