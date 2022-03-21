use crate::assemble::{Assembled, Assembler};
use crate::binary::{AccessType, BinRef, Binary, BinaryError};
use ego_tree::iter::Children;
use std::collections::{HashMap, HashSet};
use utils::sources::{ItemType, SourceDatabase, SourceMapping, Sources, SymbolError, SymbolWriter};

use crate::ast::{AstNodeId, AstNodeRef, AstTree};
use crate::item::{self, AddrModeParseType, IndexParseType, Item, Node};
use crate::messages::{info, messages};
use std::path::{Path, PathBuf};

use crate::error::UserError;

use crate::evaluator::{self, Evaluator};
use emu::cpu::RegEnum;
use emu::isa::Instruction;

use crate::gasm::GResult;
use crate::util::{ByteSize, ByteSizes};
use item::Item::*;
use crate::fixerupper::FixerUpper;
use crate::asmctx::AsmCtx;

/// Ast tree sizer
/// gets the size of everything
/// assigns values to labels that
/// are defined by value of PC

pub struct Sizer<'a> {
    offset: isize,
    tree: &'a AstTree,
}

impl<'a> Sizer<'a> {

    ////////////////////////////////////////////////////////////////////////////////
    // Node retrieval
    fn get_node(&self, id: AstNodeId) -> AstNodeRef {
        self.tree.get(id).expect("Internal error: unable to get node")
    }

    pub fn new(tree: &'a AstTree) -> Self {
        Self {
            offset: 0,
            tree,
        }
    }

    pub fn size(
        &self,
        ctx: &mut AsmCtx,
        id: AstNodeId,
    ) -> GResult<()> {
        let _ = self.size_node(ctx, 0, id)?;
        Ok(())
    }

    fn size_indexed(
        &'a self,
        ctx_mut: &'a mut AsmCtx,
        mut pc: usize,
        id: AstNodeId,
    ) -> GResult<usize> {

        let eval = &ctx_mut.eval;
        let fixer_upper = &mut ctx_mut.fixer_upper;

        let this_i = &self.get_node(id).value().item;

        let i = fixer_upper.get_fixup_or_default(pc, id, this_i);

        if let OpCode(ins, AddrModeParseType::Indexed(pmode, indirect)) = i {
            let this_pc = pc;

            pc += ins.size as usize;
            use item::IndexParseType::*;

            match pmode.clone() {
                Zero(..) | AddA(..) | AddB(..) | AddD(..) | Plus(..) | PlusPlus(..) | Sub(..)
                | SubSub(..) => (),

                ConstantByteOffset(..)
                | PcOffsetByte(..)
                | PcOffsetWord(..)
                | ConstantWordOffset(..)
                | Constant5BitOffset(..) => {}

                ConstantOffset(r) => {
                    let node = self.get_node(id);
                    let (v, _) = eval.eval_first_arg(node)?;

                    let mut bs = v.byte_size();

                    if let ByteSizes::Bits5(val) = bs {
                        if indirect {
                            // Indirect constant offset does not support
                            // 5 bit offsets so promote to 8 bit
                            bs = ByteSizes::Byte(val);
                        }
                    }

                    let new_amode = match bs {
                        ByteSizes::Zero => Zero(r),
                        ByteSizes::Bits5(v) => Constant5BitOffset(r, v),
                        ByteSizes::Word(v) => {
                            pc += 2;
                            ConstantWordOffset(r, v)
                        }
                        ByteSizes::Byte(v) => {
                            pc += 1;
                            ConstantByteOffset(r, v)
                        }
                    };

                    let new_item =
                        OpCode(ins.clone(), AddrModeParseType::Indexed(new_amode, indirect));
                    fixer_upper.add_fixup(this_pc, id, new_item);
                }

                PCOffset => {
                    let (v, id) = eval.eval_first_arg(self.get_node(id))?;
                    pc += 1;

                    let new_amode = match v.byte_size() {
                        ByteSizes::Zero => PcOffsetByte(0),
                        ByteSizes::Bits5(v) | ByteSizes::Byte(v) => PcOffsetByte(v),
                        ByteSizes::Word(v) => {
                            pc += 1;
                            PcOffsetWord(v)
                        }
                    };

                    let new_item =
                        OpCode(ins.clone(), AddrModeParseType::Indexed(new_amode, indirect));
                    fixer_upper.add_fixup(this_pc, id, new_item);
                }

                ExtendedIndirect => pc += 2,
            };
            return Ok(pc);
        }
        panic!()
    }

    fn size_node(
        &self,
        ctx: &mut AsmCtx,
        mut pc: usize,
        id: AstNodeId,
    ) -> GResult<usize> {
        use item::Item::*;

        let i = self.get_node(id).value().item.clone();

        match &i {
            MacroCallProcessed { scope, macro_id } => {
                let kids: Vec<_> = self
                    .get_node(*macro_id)
                    .children()
                    .map(|n| n.id())
                    .collect();

                ctx.eval_macro_args(scope, id, *macro_id, &self.tree);

                for c in kids {
                    pc = self.size_node(ctx, pc, c)?;
                }

                ctx.pop_scope();
            }

            Scope(opt) => {
                ctx.set_scope(opt);
            }

            GrabMem => {
                let node = self.get_node(id);
                let args = ctx.eval.eval_n_args(node, 2)?;
                let size = args[1];
                pc += size as usize;
            }

            Org => {
                let res = ctx.eval.eval_first_arg(self.get_node(id));
                if let Err(_) = res {
                    panic!();
                };

                let (value, _) = res?;
                pc = value as usize;
                ctx.fixer_upper.add_fixup(pc, id, Item::SetPc(pc));
            }

            SetPc(val) => pc = *val,

            Put => {
                let (value, _) = ctx.eval.eval_first_arg(self.get_node(id))?;
                let offset = (value - pc as i64) as isize;
                ctx.fixer_upper
                    .add_fixup(pc, id, Item::SetPutOffset(offset));
            }

            SetPutOffset(_offset) => {}

            Rmb => {
                let node = self.get_node(id);
                let (bytes, _) = ctx.eval.eval_first_arg(node)?;

                if bytes < 0 {
                    return Err(ctx
                        .eval
                        .user_error("Argument for RMB must be positive", node, true)
                        .into());
                };

                ctx.fixer_upper
                    .add_fixup(pc, id, Item::Skip(bytes as usize));

                pc += bytes as usize;
            }

            OpCode(ins, amode) => {
                use emu::isa::AddrModeEnum;

                match amode {
                    AddrModeParseType::Extended(false) => {
                        // If there is a direct page set AND
                        // I can evaluate the arg AND
                        // the instruction supports DIRECT addressing (phew)
                        // I can changing this to a direct page mode instruction
                        // !!!! and it wasn't forced (need someway to propogate this from parse)

                        let mut size = ins.size;

                        use crate::opcodes::get_opcode_info;

                        let dp_info = get_opcode_info(ins)
                            .and_then(|i_type| i_type.get_instruction(&AddrModeEnum::Direct))
                            .and_then(|ins| ctx.direct_page.map(|dp| (ins, dp)));

                        if let Some((new_ins, dp)) = dp_info {
                            if let Ok((value, _)) = ctx.eval.eval_first_arg(self.get_node(id)) {
                                let top_byte = ((value >> 8) & 0xff) as u8;

                                if top_byte == dp {
                                    // Here we go!
                                    let new_ins = new_ins.clone();
                                    size = new_ins.size;
                                    let new_item = OpCode(new_ins, AddrModeParseType::Direct);
                                    ctx.fixer_upper.add_fixup(pc, id, new_item);
                                }
                            }
                        }

                        pc += size;
                    }

                    AddrModeParseType::Indexed(..) => {
                        pc = self.size_indexed(ctx, pc, id)?;
                    }

                    _ => {
                        pc += ins.size as usize;
                    }
                };
            }

            AssignmentFromPc(name) => {
                let node = self.get_node(id);

                let pcv = if let Some(_) = node.first_child() {
                    ctx.set_pc_symbol(pc).unwrap();

                    let node = self.get_node(id);
                    let (ret, _) = ctx.eval.eval_first_arg(node)?;
                    ctx.remove_pc_symbol();
                    ret
                } else {
                    pc as i64
                };

                ctx.add_symbol_with_value(name, pcv as usize).map_err(|e| {
                    let err = if let SymbolError::Mismatch { expected } = e {
                        format!(
                            "Mismatch symbol {name} : expected {:04X} got : {:04X}",
                            expected, pcv
                        )
                    } else {
                        format!("{:?}", e)
                    };
                    ctx.eval.user_error(err, self.get_node(id), false)
                })?;
            }

            Block | TokenizedFile(..) => {
                for c in ctx.eval.get_children(self.get_node(id)) {
                    pc = self.size_node(ctx, pc, c)?;
                }
            }

            Fdb(num_of_words) => {
                pc += (*num_of_words * 2) as usize;
            }

            Fcb(num_of_bytes) => {
                pc += *num_of_bytes as usize;
            }

            Fcc(text) => {
                pc += text.as_bytes().len();
            }

            Zmb => {
                let (v, _) = ctx.eval.eval_first_arg(self.get_node(id))?;
                assert!(v >= 0);
                pc += v as usize;
            }

            Zmd => {
                let (v, _) = ctx.eval.eval_first_arg(self.get_node(id))?;
                assert!(v >= 0);
                pc += (v * 2) as usize;
            }

            Fill => {
                let (_, c) = ctx.eval.eval_two_args(self.get_node(id))?;
                assert!(c >= 0);
                pc += c as usize;
            }

            SetDp => {
                let (dp, _) = ctx.eval.eval_first_arg(self.get_node(id))?;
                ctx.set_dp(dp);
            }

            IncBin(file_name) => {
                let r = ctx
                    .eval
                    .get_binary_extents(file_name.to_path_buf(), self.get_node(id))?;
                let new_item = IncBinResolved {
                    file: file_name.to_path_buf(),
                    r: r.clone(),
                };

                ctx.fixer_upper.add_fixup(pc, id, new_item);
                pc += r.len();
            }

            WriteBin(..) | IncBinRef(..) | Assignment(..) | Comment(..) | StructDef(..)
            | MacroDef(..) | MacroCall(..) => (),
            _ => {
                let node = self.get_node(id);
                let i = &node.value().item;
                let msg = format!("Unable to size {:?}", i);
                return Err(ctx.eval.user_error(msg, node, true).into());
            }
        };

        Ok(pc)
    }

}
