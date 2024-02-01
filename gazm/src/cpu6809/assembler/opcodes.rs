use crate::{assembler::BinaryError::*, error::GResult, semantic::AstNodeId};

use crate::cpu6809::{
    frontend::{AddrModeParseType, IndexParseType, NodeKind6809},
    regutils::{reg_pair_to_flags, registers_to_flags},
    Assembler, Compiler,
};

use emu6809::isa;
use grl_sources::ItemType;

pub fn compile_indexed(
    compiler: &mut Compiler,
    asm: &mut Assembler,
    id: AstNodeId,
    imode: IndexParseType,
    indirect: bool,
) -> GResult<()> {
    use IndexParseType::*;
    let idx_byte = imode.get_index_byte(indirect);

    compiler.write_byte(idx_byte, asm, id)?;

    let node = compiler.get_node(id);

    match imode {
        PCOffset | ConstantOffset(..) => {
            panic!("Should not happen")
        }

        ExtendedIndirect => {
            let (val, _) = asm.eval_first_arg(node, compiler.scopes.scope())?;

            let res = asm.get_binary_mut().write_uword_check_size(val);
            compiler.binary_error_map(asm, id, res)?;
        }

        ConstantWordOffset(_, val) | PcOffsetWord(val) => {
            let res = asm.get_binary_mut().write_iword_check_size(val as i64);
            compiler.binary_error_map(asm, id, res)?;
        }

        ConstantByteOffset(_, val) | PcOffsetByte(val) => {
            let res = asm.get_binary_mut().write_ibyte_check_size(val as i64);
            compiler.binary_error_map(asm, id, res)?;
        }
        _ => (),
    }

    Ok(())
}
/// Compile a node
pub fn compile_node(
    compiler: &mut Compiler,
    asm: &mut Assembler,
    id: AstNodeId,
    node_kind: NodeKind6809,
) -> GResult<()> {
    use NodeKind6809::*;
    match node_kind {
        OpCode(_, ins, amode) => {
            compile_opcode(compiler, asm, id, &ins, amode)?;
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
    compiler: &mut Compiler,
    asm: &mut Assembler,
    id: AstNodeId,
    ins: &isa::Instruction,
    amode: AddrModeParseType,
) -> GResult<()> {
    use isa::AddrModeEnum;

    let pc = asm.get_binary().get_write_address();
    let ins_amode = ins.addr_mode;
    let current_scope_id = compiler.scopes.scope();

    if ins.opcode > 0xff {
        compiler.write_word(ins.opcode as u16, asm, id)
    } else {
        compiler.write_byte(ins.opcode as u8, asm, id)
    }?;

    let node = compiler.get_node(id);

    match ins_amode {
        AddrModeEnum::Indexed => {
            if let AddrModeParseType::Indexed(imode, indirect) = amode {
                compile_indexed(compiler, asm, id, imode, indirect)?;
            }
        }

        AddrModeEnum::Immediate8 => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            compiler.write_byte_check_size(arg, asm, id)?
        }

        AddrModeEnum::Direct => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            compiler.write_byte_check_size(arg & 0xff, asm, id)?
        }

        AddrModeEnum::Extended | AddrModeEnum::Immediate16 => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            compiler.write_word_check_size(arg, asm, id)?;
        }

        AddrModeEnum::Relative => {
            let (arg, arg_id) = asm.eval_first_arg(node, current_scope_id)?;
            let arg_n = compiler.get_node(arg_id);
            let val = arg - (pc as i64 + ins.size as i64);
            // offset is from PC after Instruction and operand has been fetched
            let res = asm
                .asm_out
                .binary
                .write_ibyte_check_size(val)
                .map_err(|x| match x {
                    DoesNotFit { .. } => compiler.relative_error(asm, id, val, 8),
                    DoesNotMatchReference { .. } => compiler.binary_error(asm, id, x),
                    _ => asm.make_user_error(format!("{x:?}"), arg_n, false).into(),
                });

            match &res {
                Ok(_) => (),
                Err(_) => {
                    if asm.opts.ignore_relative_offset_errors {
                        // messages::warning("Skipping writing relative offset");
                        let res = asm.get_binary_mut().write_ibyte_check_size(0);
                        compiler.binary_error_map(asm, id, res)?;
                    } else {
                        res?;
                    }
                }
            }
        }

        AddrModeEnum::Relative16 => {
            let (arg, arg_id) = asm.eval_first_arg(node, current_scope_id)?;

            let arg_n = compiler.get_node(arg_id);

            let val = (arg - (pc as i64 + ins.size as i64)) & 0xffff;
            // offset is from PC after Instruction and operand has been fetched
            let res = asm.get_binary_mut().write_word_check_size(val);

            res.map_err(|x| match x {
                DoesNotFit { .. } => compiler.relative_error(asm, id, val, 16),
                DoesNotMatchReference { .. } => compiler.binary_error(asm, id, x),
                _ => asm.make_user_error(format!("{x:?}"), arg_n, true).into(),
            })?;
        }

        AddrModeEnum::Inherent => {}

        AddrModeEnum::RegisterPair => {
            if let AddrModeParseType::RegisterPair(a, b) = amode {
                let val = reg_pair_to_flags(a, b);
                compiler.write_byte(val, asm, id)?;
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
                compiler.write_byte(flags, asm, id)?;
            } else {
                panic!()
            }
        }
    };

    // Add memory to source code mapping for this opcode
    let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
    compiler.add_mapping(asm, phys_range, range, id, ItemType::OpCode);
    Ok(())
}
