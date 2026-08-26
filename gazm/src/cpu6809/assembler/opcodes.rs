use crate::{
    assembler::{
        write_eval_byte, write_eval_word, write_opcode, write_relative_byte, write_relative_word,
        write_signed_byte, Assembler,
    },
    error::GResult,
    semantic::AstNodeRef,
};

use crate::cpu6809::{
    frontend::{AddrModeParseType, IndexParseType, NodeKind6809},
    regutils::{reg_pair_to_flags, registers_to_flags},
};

use crate::cpu6809::assembler::ISA_DBASE;
use emu6809::isa;

pub fn compile_indexed(
    asm: &mut Assembler,
    node: AstNodeRef,
    imode: IndexParseType,
    indirect: bool,
    current_scope_id: u64,
) -> GResult<()> {
    use IndexParseType::*;
    let idx_byte = imode.get_index_byte(indirect);

    asm.write_byte(idx_byte, node)?;

    match imode {
        PCOffset | ConstantOffset(..) => {
            panic!("Should not happen")
        }

        ExtendedIndirect => {
            let (val, _) = asm.eval_first_arg(node, current_scope_id)?;

            let res = asm.get_binary_mut().write_uword_check_size(val);
            asm.binary_error_map(node, res)?;
        }

        ConstantWordOffset(_, val) | PcOffsetWord(val) => {
            let res = asm.get_binary_mut().write_iword_check_size(val as i64);
            asm.binary_error_map(node, res)?;
        }

        ConstantByteOffset(_, val) | PcOffsetByte(val) => {
            write_signed_byte(asm, node, val as i64)?;
        }
        _ => (),
    }

    Ok(())
}
/// Compile a node
pub fn compile_node(
    asm: &mut Assembler,
    node: AstNodeRef,
    node_kind: NodeKind6809,
    current_scope_id: u64,
) -> GResult<()> {
    use NodeKind6809::*;
    match node_kind {
        OpCode(ins, amode) => {
            compile_opcode(asm, node, ISA_DBASE.get_by_id(ins), amode, current_scope_id)?;
        }

        SetDp => {
            let (dp, _) = asm.eval_first_arg(node, current_scope_id)?;
            if !(0..=0xff).contains(&dp) {
                return Err(asm
                    .make_user_error("SETDP value must be between 0 and 255", node, true)
                    .into());
            }
            asm.asm_out.set_dp(dp as u8);
        }

        Illegal => todo!(),
        Operand(_) => todo!(),
        OperandIndexed(_, _) => todo!(),
        RegisterSet(_) => todo!(),
    }
    Ok(())
}

/// Compile an opcode: the opcode value comes from the instruction row and
/// the operand bytes come from the shared writers, keyed by the row's
/// addressing mode. Only the modes with CPU-specific encoding stay here.
pub fn compile_opcode(
    asm: &mut Assembler,
    node: AstNodeRef,
    ins: &isa::Instruction,
    amode: AddrModeParseType,
    current_scope_id: u64,
) -> GResult<()> {
    use isa::AddrModeEnum;

    let pc = asm.get_binary().get_write_address();

    write_opcode(asm, node, ins.opcode)?;

    match ins.addr_mode {
        AddrModeEnum::Indexed => {
            if let AddrModeParseType::Indexed(imode, indirect) = amode {
                compile_indexed(asm, node, imode, indirect, current_scope_id)?;
            }
            Ok(())
        }

        AddrModeEnum::Immediate8 => write_eval_byte(asm, node, current_scope_id),

        AddrModeEnum::Direct => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            asm.write_byte_check_size(arg & 0xff, node)?;
            Ok(())
        }

        AddrModeEnum::Extended | AddrModeEnum::Immediate16 => {
            write_eval_word(asm, node, current_scope_id)
        }

        AddrModeEnum::Relative => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            write_relative_byte(asm, node, arg, pc, ins.size)
        }

        AddrModeEnum::Relative16 => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            write_relative_word(asm, node, arg, pc, ins.size)
        }

        AddrModeEnum::Inherent => Ok(()),

        AddrModeEnum::RegisterPair => {
            if let AddrModeParseType::RegisterPair(a, b) = amode {
                let val = reg_pair_to_flags(a, b);
                asm.write_byte(val, node)?;
            } else {
                panic!("Whut!")
            }
            Ok(())
        }

        AddrModeEnum::RegisterSet => {
            use crate::frontend::{AstNodeKind, CpuSpecific::Cpu6809};
            use NodeKind6809::RegisterSet;
            let rset = &node.first_child().unwrap().value().item;

            if let AstNodeKind::TargetSpecific(Cpu6809(RegisterSet(regs))) = &rset {
                let flags = registers_to_flags(regs);
                asm.write_byte(flags, node)?;
            } else {
                panic!()
            }
            Ok(())
        }
    }
}
