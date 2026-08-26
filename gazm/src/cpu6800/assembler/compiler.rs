use crate::{cpu6800::frontend::NodeKind6800, semantic::AstNodeRef};

use emu6800::cpu_core::{AddrModeEnum, InstructionInfo, DBASE};

use crate::{
    assembler::{write_eval_byte, write_eval_word, write_opcode, write_relative_byte, Assembler},
    error::GResult,
};

impl Assembler {
    /// Compile a 6800 opcode: the opcode byte plus the operand bytes via
    /// the shared writers, keyed by the row's addressing mode. Reference
    /// mismatches fail the build like every other backend (the old
    /// "Waring!" swallow silently emitted divergent bytes).
    pub fn compile_opcode_6800(
        &mut self,
        node: AstNodeRef,
        ins: InstructionInfo,
        current_scope_id: u64,
    ) -> GResult<()> {
        let pc = self.get_binary().get_write_address();
        write_opcode(self, node, ins.opcode_data.opcode)?;

        match ins.addr_mode {
            AddrModeEnum::Indexed | AddrModeEnum::Direct | AddrModeEnum::Immediate8 => {
                write_eval_byte(self, node, current_scope_id)
            }

            AddrModeEnum::Extended | AddrModeEnum::Immediate16 => {
                write_eval_word(self, node, current_scope_id)
            }

            AddrModeEnum::Inherent => Ok(()),

            AddrModeEnum::Relative => {
                let (arg, _) = self.eval_first_arg(node, current_scope_id)?;
                write_relative_byte(self, node, arg, pc, ins.opcode_data.size)
            }

            AddrModeEnum::Illegal => todo!(),
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
            OpCode(ins, _amode) => {
                let ins = DBASE.get_instruction_info_from_opcode(ins.0).unwrap();
                self.compile_opcode_6800(node, ins, current_scope_id)?;
            }

            Illegal => todo!("Illegal"),
            Operand(_) => todo!("Operand!"),
        }

        Ok(())
    }
}
