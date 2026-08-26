use crate::{cpu6800::frontend::NodeKind6800, semantic::AstNodeRef};

use emu6800::cpu_core::{AddrModeEnum, InstructionInfo, DBASE};

use crate::{
    assembler::{write_eval_byte, write_eval_word, write_opcode, write_relative_byte, Assembler},
    error::GResult,
};

/// Compile a 6800 opcode: the opcode byte plus the operand bytes via
/// the shared writers, keyed by the row's addressing mode. Reference
/// mismatches fail the build like every other backend (the old
/// "Waring!" swallow silently emitted divergent bytes).
pub fn compile_opcode_6800(
    asm: &mut Assembler,
    node: AstNodeRef,
    ins: InstructionInfo,
    current_scope_id: u64,
) -> GResult<()> {
    let pc = asm.get_binary().get_write_address();
    write_opcode(asm, node, ins.opcode_data.opcode)?;

    match ins.addr_mode {
        AddrModeEnum::Indexed | AddrModeEnum::Direct | AddrModeEnum::Immediate8 => {
            write_eval_byte(asm, node, current_scope_id)
        }

        AddrModeEnum::Extended | AddrModeEnum::Immediate16 => {
            write_eval_word(asm, node, current_scope_id)
        }

        AddrModeEnum::Inherent => Ok(()),

        AddrModeEnum::Relative => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            write_relative_byte(asm, node, arg, pc, ins.opcode_data.size)
        }

        AddrModeEnum::Illegal => todo!(),
    }
}

/// Compile a node (registry entry: `crate::cpu6800::assembler::compile_node`).
pub fn compile_node(
    asm: &mut Assembler,
    node: AstNodeRef,
    node_kind: NodeKind6800,
    current_scope_id: u64,
) -> GResult<()> {
    use NodeKind6800::*;

    match node_kind {
        OpCode(ins, _amode) => {
            let ins = DBASE.get_instruction_info_from_opcode(ins.0).unwrap();
            compile_opcode_6800(asm, node, ins, current_scope_id)?;
        }

        Illegal => todo!("Illegal"),
        Operand(_) => todo!("Operand!"),
    }

    Ok(())
}
