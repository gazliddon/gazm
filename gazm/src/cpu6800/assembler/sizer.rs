use emu6800::cpu::ISA_DBASE;
use emu6800::cpu_core::{AddrModeEnum, OpcodeId, DBASE};

use crate::debug_mess;
use crate::{
    assembler::{Assembler, Sizer},
    error::GResult,
    semantic::AstNodeId,
};

use crate::cpu6800::frontend::{AddrModeParseType, NodeKind6800};

/// Size a 6800 node (registry entry: `crate::cpu6800::assembler::size_node_internal`).
pub fn size_node_internal(
    sizer: &mut Sizer,
    asm: &mut Assembler,
    id: AstNodeId,
    node_kind: NodeKind6800,
) -> GResult<()> {
    let current_scope_id = sizer.scopes.scope();
    use NodeKind6800::*;

    match &node_kind {
        Illegal => todo!(),

        OpCode(ins, amode) => {
            let opcode_id = *ins;
            let ins_info = ISA_DBASE
                .get_instruction_info_from_opcode(opcode_id.0)
                .unwrap();
            let ins = ins_info.opcode_data;
            // get the size of this instruction
            let mut size = ins.size;

            let forced_extended = *amode == AddrModeParseType::Extended(true);

            if ins_info.addr_mode == AddrModeEnum::Extended && !forced_extended {
                // Is this extend addressing and we support direct?
                // If see evaluate the operand and see if the result is
                // in the first page
                // If it is we can do direct addressing

                let node = sizer.get_node(id);
                let value = asm
                    .eval_first_arg(node, current_scope_id)
                    .ok()
                    .map(|(v, _)| v);

                if let Some(new_size) = crate::assembler::try_direct_page(
                    Some(0),
                    value,
                    || {
                        DBASE
                            .get_instruction_info_from_opcode(opcode_id.0)
                            .and_then(|i_type| {
                                i_type.instruction.get_opcode_data(AddrModeEnum::Direct)
                            })
                            .map(|i| (i.size, i.id().0))
                    },
                    |new_id| {
                        let new_item = OpCode(OpcodeId(new_id), AddrModeParseType::Direct);
                        sizer.set_node_fixup(id, new_item);
                    },
                ) {
                    let src = asm.get_source_info(&node.value().pos);

                    if let Ok(src) = src {
                        debug_mess!("Xformed from Extended to Direct :  {}", src.line_str);
                    }

                    size = new_size;
                }
            }

            sizer.advance_pc(size)
        }

        Operand(_) => todo!(),
    }
    Ok(())
}
