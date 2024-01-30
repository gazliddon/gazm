use crate::cpu6800::{frontend::MC6800, AddrModeParseType, Assembler, Compiler};

use emu6800::cpu_core::{OpcodeData, DBASE};

use crate::{error::GResult, semantic::AstNodeId, assembler::BinaryError};

/// Compile an opcode
pub fn compile_opcode(
    compiler: &mut Compiler,
    asm: &mut Assembler,
    id: AstNodeId,
    ins: &OpcodeData,
    _amode: AddrModeParseType,
) -> GResult<()> {
    let opcode = ins.opcode;
    let size = ins.size;
    let ins = DBASE.get_instruction_info_from_opcode(ins.opcode).unwrap();
    let current_scope_id = compiler.scopes.scope();

    compiler.write_byte(opcode as u8, asm, id)?;

    let node = compiler.get_node(id);

    use emu6800::cpu_core::AddrModeEnum;

    match ins.addr_mode {
        AddrModeEnum::Indexed | AddrModeEnum::Direct | AddrModeEnum::Immediate8 => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            compiler.write_byte_check_size(arg, asm, id)?
        }

        AddrModeEnum::Extended | AddrModeEnum::Immediate16 => {
            let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
            compiler.write_word_check_size(arg, asm, id)?;
        }

        AddrModeEnum::Inherent => (),

        AddrModeEnum::Relative => {

            use BinaryError::*;

            let pc = asm.get_binary().get_write_address();
            let (arg, arg_id) = asm.eval_first_arg(node, current_scope_id)?;
            let arg_n = compiler.get_node(arg_id);
            let val = arg - (pc as i64 + size as i64);
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

        AddrModeEnum::Illegal => todo!(),
    };
    Ok(())
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
        OpCode(_, ins, amode) => {
            compile_opcode(compiler, asm, id, &ins, amode)?;
        }

        Illegal => todo!("Illegal"),
        Operand(_) => todo!("Operand!"),
    }

    Ok(())
}
