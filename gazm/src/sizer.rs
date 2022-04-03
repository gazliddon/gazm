use crate::binary::{AccessType, BinRef, Binary, BinaryError};
use ego_tree::iter::Children;
use serde_json::ser::Formatter;
use std::collections::{HashMap, HashSet};
use utils::sources::{
    ItemType, SourceDatabase, SourceMapping, Sources, SymbolError, SymbolQuery, SymbolWriter,
};

use crate::ast::{AstNodeId, AstNodeRef, AstTree};
use crate::item::{self, AddrModeParseType, IndexParseType, Item, Node};
use crate::messages::{info, messages};
use std::path::Path;

use crate::error::UserError;

use crate::evaluator::{self, Evaluator};
use emu::cpu::RegEnum;
use emu::isa::Instruction;

use crate::asmctx::AsmCtx;
use crate::error::{GResult, GazmError};
use crate::fixerupper::FixerUpper;
use crate::util::{ByteSize, ByteSizes};
use item::Item::*;

/// Ast tree sizer
/// gets the size of everything
/// assigns values to labels that
/// are defined by value of PC
struct Sizer<'a> {
    offset: isize,
    tree: &'a AstTree,
}

pub fn size_tree(ctx: &mut AsmCtx, id: AstNodeId, tree : &AstTree) -> GResult<()> {
    let sizer = Sizer::new(tree);
    ctx.set_root_scope();
    let _ = sizer.size_node(ctx, 0, id)?;
    Ok(())
}

impl<'a> Sizer<'a> {
    pub fn new(tree: &'a AstTree) -> Self {
        Self { offset: 0, tree }
    }

    pub fn size(&self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        ctx.set_root_scope();
        let _ = self.size_node(ctx, 0, id)?;
        Ok(())
    }

    fn size_indexed(&self, ctx_mut: &mut AsmCtx, mut pc: usize, id: AstNodeId) -> GResult<usize> {
        let eval = &ctx_mut.eval;

        let (node, i) = self.get_node_item(ctx_mut, id);

        if let OpCode(ins, AddrModeParseType::Indexed(pmode, indirect)) = i {
            let _this_pc = pc;

            pc += ins.size as usize;
            use item::IndexParseType::*;

            match pmode {
                Zero(..) | AddA(..) | AddB(..) | AddD(..) | Plus(..) | PlusPlus(..) | Sub(..)
                | SubSub(..) => (),

                ConstantByteOffset(..)
                | PcOffsetByte(..)
                | PcOffsetWord(..)
                | ConstantWordOffset(..)
                | Constant5BitOffset(..) => {}

                ConstantOffset(r) => {
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
                        OpCode(ins, AddrModeParseType::Indexed(new_amode, indirect));

                    ctx_mut.add_fixup(id, new_item);
                }

                PCOffset => {
                    let (v, _) = eval.eval_first_arg(node)?;
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
                        OpCode(ins, AddrModeParseType::Indexed(new_amode, indirect));

                    ctx_mut.add_fixup(id, new_item);
                }

                ExtendedIndirect => pc += 2,
            };
            return Ok(pc);
        }
        panic!()
    }

    fn size_node(&self, ctx: &mut AsmCtx, mut pc: usize, id: AstNodeId) -> GResult<usize> {
        use item::Item::*;

        let (node, i) = self.get_node_item(ctx, id);

        match &i {
            MacroCallProcessed { scope, macro_id } => {
                ctx.eval_macro_args(scope, id, *macro_id, self.tree);

                ctx.set_scope(scope);

                let (m_node, _) = self.get_node_item(ctx, *macro_id);

                let kids: Vec<_> = m_node.children().map(|n| n.id()).collect();

                for c in kids {
                    pc = self.size_node(ctx, pc, c)?;
                }

                ctx.pop_scope();
            }

            Scope(opt) => {
                ctx.set_root_scope();
                if opt != "root" {
                    ctx.set_scope(opt);
                }
            }

            GrabMem => {
                let args = ctx.eval.eval_n_args(node, 2)?;
                let size = args[1];
                pc += size as usize;
            }

            Org => {
                let res = ctx.eval.eval_first_arg(node);
                if res.is_err() {
                    panic!();
                };

                let (value, _) = res?;
                pc = value as usize;
                ctx.add_fixup(id, Item::SetPc(pc));
            }

            SetPc(val) => pc = *val,

            Put => {
                let (value, _) = ctx.eval.eval_first_arg(node)?;
                let offset = (value - pc as i64) as isize;
                ctx.add_fixup(id, Item::SetPutOffset(offset));
            }

            Rmb => {
                let (bytes, _) = ctx.eval.eval_first_arg(node)?;

                if bytes < 0 {
                    return Err(ctx
                        .eval
                        .user_error("Argument for RMB must be positive", node, true)
                        .into());
                };

                ctx.add_fixup(id, Item::Skip(bytes as usize));

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
                            if let Ok((value, _)) = ctx.eval.eval_first_arg(node) {
                                let top_byte = ((value >> 8) & 0xff) as u8;

                                if top_byte == dp {
                                    // Here we go!
                                    let new_ins = new_ins.clone();
                                    size = new_ins.size;
                                    let new_item = OpCode(new_ins, AddrModeParseType::Direct);
                                    ctx.add_fixup(id, new_item);
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
                // TODO should two types of item rather than this
                // conditional
                let pcv = if node.first_child().is_some() {
                    // Assign this label
                    // If the label has a child it means
                    // assignment is from an expr containing the current PC
                    // so lets evaluate it!
                    ctx.set_pc_symbol(pc).unwrap();
                    let (ret, _) = ctx.eval.eval_first_arg(node)?;
                    ctx.remove_pc_symbol();
                    ret
                } else {
                    // Otherwise it's just the current PC
                    pc as i64
                };

                let scope = ctx.get_scope_fqn();
                let msg = format!("Setting {scope}::{name} to ${pc:04X} ({pc})");
                messages().debug(msg);

                // Add the symbol
                ctx.add_symbol_with_value(name, pcv as usize).map_err(|e| {
                    let err = if let SymbolError::Mismatch { expected } = e {
                        format!(
                            "Mismatch symbol {name} : expected {:04X} got : {:04X}",
                            expected, pcv
                        )
                    } else {
                        let z = ctx
                            .eval
                            .get_symbols()
                            .get_symbol_info(name)
                            .unwrap()
                            .clone();
                        let scope = ctx.get_scope_fqn();
                        format!("Scope: {scope} {z:#?} - {:?}", e)
                    };
                    ctx.eval.user_error(err, node, false)
                })?;
            }

            Block | TokenizedFile(..) => {
                for c in ctx.eval.get_children(node) {
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
                let (v, _) = ctx.eval.eval_first_arg(node)?;
                assert!(v >= 0);
                pc += v as usize;
            }

            Zmd => {
                let (v, _) = ctx.eval.eval_first_arg(node)?;
                assert!(v >= 0);
                pc += (v * 2) as usize;
            }

            Fill => {
                let (_, c) = ctx.eval.eval_two_args(node)?;
                assert!(c >= 0);
                pc += c as usize;
            }

            SetDp => {
                let (dp, _) = ctx.eval.eval_first_arg(node)?;
                ctx.set_dp(dp);
            }

            IncBin(file_name) => {
                let r = self.get_binary_extents(ctx, file_name, node)?;
                let new_item = IncBinResolved {
                    file: file_name.to_path_buf(),
                    r: r.clone(),
                };

                ctx.add_fixup(id, new_item);
                pc += r.len();
            }

            PostFixExpr | WriteBin(..) | IncBinRef(..) | Assignment(..) | Comment(..)
            | StructDef(..) | MacroDef(..) | MacroCall(..) => (),
            _ => {
                let msg = format!("Unable to size {:?}", i);
                return Err(ctx.eval.user_error(msg, node, true).into());
            }
        };

        Ok(pc)
    }

    fn get_binary_extents<P: AsRef<Path>>(
        &self,
        ctx: &mut AsmCtx,
        file_name: P,
        node: AstNodeRef,
    ) -> GResult<std::ops::Range<usize>> {
        use utils::sources::FileIo;
        let data_len = ctx.get_file_size(file_name.as_ref())?;

        let mut r = 0..data_len;

        let mut c = node.children();

        let offset_size = c
            .next()
            .and_then(|offset| c.next().map(|size| (offset, size)));

        if let Some((offset, size)) = offset_size {
            let offset = ctx.eval.eval_node(offset)?;
            let size = ctx.eval.eval_node(size)?;
            let offset = offset as usize;
            let size = size as usize;
            let last = (offset + size) - 1;

            if !(r.contains(&offset) && r.contains(&last)) {
                let msg =
                    format!("Trying to grab {offset:04X} {size:04X} from file size {data_len:X}");
                return Err(ctx.eval.user_error(msg, node, true).into());
            };

            r.start = offset;
            r.end = offset + size;
        }

        Ok(r)
    }

    fn get_node_item(&self, ctx: &AsmCtx, id: AstNodeId) -> (AstNodeRef, Item) {
        let node = self.tree.get(id).unwrap();
        let this_i = &node.value().item;
        let i = ctx.get_fixup_or_default(id, this_i);
        (node, i)
    }
}

