use crate::{cpu6800::frontend::NodeKind6800, semantic::AstNodeRef, };

use emu6800::cpu_core::{AddrModeEnum, InstructionInfo, DBASE};

use crate::{
    assembler::{Assembler, BinaryError},
    error::{GResult, GazmErrorKind},
};
/// Compile an opcode

impl Assembler {
    pub fn compile_operand_6800(
        &mut self,
        node: AstNodeRef,
        ins: InstructionInfo,
        pc: i64,
        current_scope_id: u64,
    ) -> GResult<()> {
        match ins.addr_mode {
            AddrModeEnum::Indexed | AddrModeEnum::Direct | AddrModeEnum::Immediate8 => {
                let (arg, _id) = self.eval_first_arg(node, current_scope_id)?;
                self.get_binary_mut().write_byte_check_size(arg)?;
            }

            AddrModeEnum::Extended => {
                let (arg, _id) = self.eval_first_arg(node, current_scope_id)?;
                self.get_binary_mut().write_word_check_size(arg)?;
            }

            AddrModeEnum::Immediate16 => {
                let (arg, _id) = self.eval_first_arg(node, current_scope_id)?;
                self.get_binary_mut().write_word_check_size(arg)?;
            }

            AddrModeEnum::Inherent => (),

            AddrModeEnum::Relative => {
                let size = ins.opcode_data.size as i64;
                let (arg, _arg_id) = self.eval_first_arg(node, current_scope_id)?;

                let val = arg - (pc + size);
                let binary = &mut self.asm_out.binary;
                binary.write_ibyte_check_size(val)?;
            }

            AddrModeEnum::Illegal => todo!(),
        };

        Ok(())
    }

    /// Compile an opcode
    pub fn compile_opcode_6800(
        &mut self,
        node: AstNodeRef,
        ins: InstructionInfo,
        current_scope_id: u64,
    ) -> GResult<()> {
        let pc = self.get_binary().get_write_address() as i64;
        self.get_binary_mut()
            .write_byte(ins.opcode_data.opcode as u8)?;

        let ret = self.compile_operand_6800(node, ins, pc, current_scope_id);

        match ret {
            Ok(()) => Ok(()),
            Err(GazmErrorKind::BinaryError(BinaryError::DoesNotMatchReference(..))) => {
                eprintln!("Waring!");
                Ok(())
            }
            Err(_) => {
                println!("ret {ret:?}");
                ret
            }
        }
    }

    /// Compile a node
    pub fn compile_node_6800(
        &mut self,
        node_kind: NodeKind6800,
        node: AstNodeRef,
        current_scope_id: u64,
    ) -> GResult<()> {
        use NodeKind6800::*;


        match node_kind {
            OpCode(_, ins) => {
                let ins = DBASE.get_instruction_info_from_opcode(ins.opcode).unwrap();
                self.compile_opcode_6800(node, ins, current_scope_id)?;
            }

            Illegal => todo!("Illegal"),
            Operand(_) => todo!("Operand!"),
        }

        Ok(())
    }
}
