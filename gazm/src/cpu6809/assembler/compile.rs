use crate::{
    assembler::{Assembler, AssemblerCpuTrait, BinaryError, Compiler, ScopeTracker, Sizer},
    error::GResult,
    frontend::{Item, Node},
    semantic::AstNodeId,
};

use grl_sources::ItemType;

use emu6809::isa;
use tower_lsp::lsp_types::notification;

use super::super::{
    frontend::{AddrModeParseType, IndexParseType, MC6809},
    reg_pair_to_flags, registers_to_flags,
};

#[derive(Debug, Clone,Copy, PartialEq, Default)]
pub struct Compiler6809 {}

impl Compiler6809 {
    pub fn new() -> Self {
        Self {}
    }

    fn compile_indexed(
        compiler: &mut Compiler<Self>,
        asm: &mut Assembler<Self>,
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

    /// Compile an opcode
    fn compile_opcode(
        compiler: &mut Compiler<Self>,
        asm: &mut Assembler<Self>,
        id: AstNodeId,
        ins: &isa::Instruction,
        amode: AddrModeParseType,
    ) -> GResult<()> {
        use isa::AddrModeEnum::*;

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
            Indexed => {
                if let AddrModeParseType::Indexed(imode, indirect) = amode {
                    Self::compile_indexed(compiler, asm, id, imode, indirect)?;
                }
            }

            Immediate8 => {
                let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
                compiler.write_byte_check_size(arg, asm, id)?
            }

            Direct => {
                let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
                compiler.write_byte_check_size(arg & 0xff, asm, id)?
            }

            Extended | Immediate16 => {
                let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
                compiler.write_word_check_size(arg, asm, id)?;
            }

            Relative => {
                use BinaryError::*;
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

            Relative16 => {
                use BinaryError::*;

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

            Inherent => {}

            RegisterPair => {
                if let AddrModeParseType::RegisterPair(a, b) = amode {
                    let val = reg_pair_to_flags(a, b);
                    compiler.write_byte(val, asm, id)?;
                } else {
                    panic!("Whut!")
                }
            }

            RegisterSet => {
                let rset = &node.first_child().unwrap().value().item;
                if let Item::CpuSpecific(MC6809::RegisterSet(regs)) = rset {
                    let flags = registers_to_flags(regs);
                    compiler.write_byte(flags, asm, id)?;
                } else {
                    panic!("Whut!")
                }
            }
        };

        // Add memory to source code mapping for this opcode
        let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
        compiler.add_mapping(asm, phys_range, range, id, ItemType::OpCode);
        Ok(())
    }
}

impl AsRef<Node<Compiler6809>> for Node<Compiler6809> {
    fn as_ref(&self) -> &Node<Compiler6809> {
        todo!()
    }
}

impl<'a> AssemblerCpuTrait for Compiler6809 {
    type Node = Node<Self>;
    type NodeKind = MC6809;

    fn new() -> Self {
        Compiler6809::new()
    }

    fn compile_node(
        compiler: &mut Compiler<Self>,
        asm: &mut Assembler<Self>,
        id: AstNodeId,
        node_kind: Self::NodeKind,
    ) -> GResult<()> {
        match node_kind {
            MC6809::OpCode(_, ins, amode) => {
                Self::compile_opcode(compiler, asm, id, &ins, amode)?;
            }
            _ => (),
        }
        Ok(())
    }

    fn size_node(
        _sizer: &mut Sizer<Self>,
        _asm: &mut Assembler<Self>,
        _id: AstNodeId,
        _node_kind: Self::NodeKind,
    ) -> GResult<()> {
        Self::size_node_internal(_sizer,_asm,_id,_node_kind)
    }
}
