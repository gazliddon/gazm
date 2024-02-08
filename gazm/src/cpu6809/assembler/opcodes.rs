use crate::{assembler::BinaryError::*, error::GResult, semantic::AstNodeRef};

use crate::assembler::Assembler;
use crate::cpu6809::{
    frontend::{AddrModeParseType, IndexParseType, NodeKind6809},
    regutils::{reg_pair_to_flags, registers_to_flags},
};

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
            let res = asm.get_binary_mut().write_ibyte_check_size(val as i64);
            asm.binary_error_map(node, res)?;
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
        OpCode(_, ins, amode) => {
            compile_opcode(asm, node, &ins, amode, current_scope_id)?;
        }

        SetDp => eprintln!("Warning! SetDP node not compiled!"),

        Illegal => todo!(),
        Operand(_) => todo!(),
        OperandIndexed(_, _) => todo!(),
        RegisterSet(_) => todo!(),
    }
    Ok(())
}

/// Compile an opcode
pub fn compile_opcode(
    asm: &mut Assembler,
    node: AstNodeRef,
    ins: &isa::Instruction,
    amode: AddrModeParseType,
    current_scope_id: u64,
) -> GResult<()> {
    use isa::AddrModeEnum;

    let pc = asm.get_binary().get_write_address();
    let ins_amode = ins.addr_mode;

    if ins.opcode > 0xff {
        asm.write_word(ins.opcode as u16, node)
    } else {
        asm.write_byte(ins.opcode as u8, node)
    }?;

    match ins_amode {
        AddrModeEnum::Indexed => {
            if let AddrModeParseType::Indexed(imode, indirect) = amode {
                compile_indexed(asm, node, imode, indirect, current_scope_id)?;
            }
        }

        AddrModeEnum::Immediate8 => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            asm.write_byte_check_size(arg, node)?
        }

        AddrModeEnum::Direct => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            asm.write_byte_check_size(arg & 0xff, node)?
        }

        AddrModeEnum::Extended | AddrModeEnum::Immediate16 => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            asm.write_word_check_size(arg, node)?;
        }

        AddrModeEnum::Relative => {
            let (arg, arg_n) = asm.eval_first_arg_n(node, current_scope_id)?;
            let val = arg - (pc as i64 + ins.size as i64);
            // offset is from PC after Instruction and operand has been fetched
            let res = asm
                .asm_out
                .binary
                .write_ibyte_check_size(val)
                .map_err(|x| match x {
                    DoesNotFit { .. } => {
                        panic!()
                    }
                    DoesNotMatchReference { .. } => asm.binary_error(node, x),
                    _ => asm.make_user_error(format!("{x:?}"), arg_n, false).into(),
                });

            match &res {
                Ok(_) => (),
                Err(_) => {
                    if asm.opts.ignore_relative_offset_errors {
                        // messages::warning("Skipping writing relative offset");
                        let res = asm.get_binary_mut().write_ibyte_check_size(0);
                        asm.binary_error_map(node, res)?;
                    } else {
                        res?;
                    }
                }
            }
        }

        AddrModeEnum::Relative16 => {
            let (arg, arg_n) = asm.eval_first_arg_n(node, current_scope_id)?;

            let val = (arg - (pc as i64 + ins.size as i64)) & 0xffff;
            // offset is from PC after Instruction and operand has been fetched
            let res = asm.get_binary_mut().write_word_check_size(val);

            res.map_err(|x| match x {
                DoesNotFit { .. } => {
                    panic!()
                }
                DoesNotMatchReference { .. } => asm.binary_error(node, x),
                _ => asm.make_user_error(format!("{x:?}"), arg_n, true).into(),
            })?;
        }

        AddrModeEnum::Inherent => {}

        AddrModeEnum::RegisterPair => {
            if let AddrModeParseType::RegisterPair(a, b) = amode {
                let val = reg_pair_to_flags(a, b);
                asm.write_byte(val, node)?;
            } else {
                panic!("Whut!")
            }
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
        }
    };

    // Add memory to source code mapping for this opcode
    // let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
    // compiler.add_mapping(asm, phys_range, range, node.id(), ItemType::OpCode);
    Ok(())
}
