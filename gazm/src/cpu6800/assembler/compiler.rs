use crate::cpu6800::{frontend::MC6800, Assembler, Compiler, DBASE};

use emu6800::cpu_core::{AddrModeEnum, InstructionInfo};

use crate::{assembler::BinaryError, error::{ GResult, GazmErrorKind }, semantic::AstNodeId};
/// Compile an opcode
pub fn compile_operand(
    compiler: &mut Compiler,
    asm: &mut Assembler,
    id: AstNodeId,
    ins: InstructionInfo,
    pc: i64,
) -> GResult<()> {
    let current_scope_id = compiler.scopes.scope();
    let node = compiler.get_node(id);

    match ins.addr_mode {
        AddrModeEnum::Indexed | AddrModeEnum::Direct | AddrModeEnum::Immediate8 => {
            let (arg, _id) = asm.eval_first_arg(node, current_scope_id)?;
                asm.get_binary_mut()
                .write_byte_check_size(arg)?;
        }

        AddrModeEnum::Extended => {
            let (arg, _id) = asm.eval_first_arg(node, current_scope_id)?;
            asm.get_binary_mut()
                .write_word_check_size(arg, )?;
        }

        AddrModeEnum::Immediate16 => {
            let (arg, _id) = asm.eval_first_arg(node, current_scope_id)?;
            asm.get_binary_mut().write_word_check_size(arg)?;
        }

        AddrModeEnum::Inherent => (),

        AddrModeEnum::Relative => {
            let size = ins.opcode_data.size as i64;
            let (arg, _arg_id) = asm.eval_first_arg(node, current_scope_id)?;

            let val = arg - (pc as i64 + size);
            let binary = &mut asm.asm_out.binary;
            binary.write_ibyte_check_size(val)?;
        }

        AddrModeEnum::Illegal => todo!(),
    };

    Ok(())
}

/// Compile an opcode
pub fn compile_opcode(
    compiler: &mut Compiler,
    asm: &mut Assembler,
    id: AstNodeId,
    ins: InstructionInfo,
) -> GResult<()> {
    let pc = asm.get_binary().get_write_address() as i64;

    compiler.write_byte(ins.opcode_data.opcode as u8, asm, id)?;

    let ret = compile_operand(compiler, asm, id, ins, pc);

    match ret {
        Ok(()) => Ok(()),
        Err(GazmErrorKind::BinaryError(BinaryError::DoesNotMatchReference(..))) => {
            eprintln!("Waring!");
            Ok(())
        },
        Err(_) => {println!("ret {ret:?}"); ret},
    }
}

/// Compile a node
pub fn compile_node(
    compiler: &mut Compiler,
    asm: &mut Assembler,
    id: AstNodeId,
    node_kind: MC6800,
) -> GResult<()> {
    use MC6800::*;

    match node_kind {
        OpCode(_, ins) => {
            let ins = DBASE.get_instruction_info_from_opcode(ins.opcode).unwrap();
            compile_opcode(compiler, asm, id, ins)?;
        }

        Illegal => todo!("Illegal"),
        Operand(_) => todo!("Operand!"),
    }

    Ok(())
}
