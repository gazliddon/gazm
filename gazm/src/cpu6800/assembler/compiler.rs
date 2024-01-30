use crate::cpu6800::{frontend::MC6800, AddrModeParseType, Assembler, Compiler};

use emu6800::cpu_core::{OpcodeData, DBASE};

use crate::{error::GResult, semantic::AstNodeId};

/// Compile an opcode
pub fn compile_opcode(
    compiler: &mut Compiler,
    asm: &mut Assembler,
    id: AstNodeId,
    ins: &OpcodeData,
    _amode: AddrModeParseType,
) -> GResult<()> {
    let opcode = ins.opcode;
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
            let _pc = asm.get_binary().get_write_address();
            compiler.write_byte_check_size(0, asm, id)?;
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
