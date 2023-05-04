use crate::{
    asmctx::AsmCtx,
    ast::{AstNodeId, AstNodeRef, AstTree},
    error::GResult,
    item::{self, Item, LabelDefinition},
    parse::util::{ByteSize, ByteSizes},
};

use crate::{
    item6809::{
        AddrModeParseType,
        MC6809::{self, OpCode, SetDp},
    },
    parse6809::opcodes::get_opcode_info,
};

use emu::{
    isa::AddrModeEnum,
    utils::sources::{SymbolScopeId, SymbolWriter},
};

use std::path::Path;

use Item::*;

/// Ast tree sizer
/// gets the size of everything
/// assigns values to labels that
/// are defined by value of PC
struct Sizer<'a> {
    _offset: isize,
    tree: &'a AstTree,
}

pub fn size_tree(ctx: &mut AsmCtx, id: AstNodeId, tree: &AstTree) -> GResult<()> {
    let sizer = Sizer::new(tree);
    ctx.set_root_scope();

    let pc_id = ctx
        .ctx
        .get_symbols_mut()
        .add_symbol_with_value("*", 0)
        .expect("Can't add symbol for pc");

    let _ = sizer.size_node(ctx, 0, id, pc_id, ctx.get_current_scope_id())?;

    ctx.set_root_scope();
    ctx.remove_symbol("*");
    Ok(())
}

impl<'a> Sizer<'a> {
    pub fn new(tree: &'a AstTree) -> Self {
        Self { _offset: 0, tree }
    }

    fn size_indexed(&self, ctx_mut: &mut AsmCtx, mut pc: usize, id: AstNodeId, current_scope_id: u64) -> GResult<usize> {
        // let eval = &ctx_mut.eval;

        let (node, i) = self.get_node_item(ctx_mut, id);

        if let Cpu(OpCode(text, ins, AddrModeParseType::Indexed(pmode, indirect))) = i {
            use crate::item6809::IndexParseType::*;
            pc += ins.size;

            match pmode {
                Zero(..) | AddA(..) | AddB(..) | AddD(..) | Plus(..) | PlusPlus(..) | Sub(..)
                | SubSub(..) => (),

                ConstantByteOffset(..)
                | PcOffsetByte(..)
                | PcOffsetWord(..)
                | ConstantWordOffset(..)
                | Constant5BitOffset(..) => {}

                ConstantOffset(r) => {
                    let (v, _) = ctx_mut.ctx.eval_first_arg(node, current_scope_id)?;

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
                        OpCode(text, ins, AddrModeParseType::Indexed(new_amode, indirect));

                    ctx_mut.add_fixup(id, new_item);
                }

                PCOffset => {
                    let (v, _) = ctx_mut.ctx.eval_first_arg(node, current_scope_id)?;
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
                        OpCode(text, ins, AddrModeParseType::Indexed(new_amode, indirect));
                    ctx_mut.add_fixup(id, new_item);
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
        pc_symbol_id: SymbolScopeId,
        current_scope_id: u64,
    ) -> GResult<usize> {
        use item::Item::*;

        let (node, i) = self.get_node_item(ctx, id);

        match &i {
            MacroCallProcessed {
                scope_id, macro_id, ..
            } => {
                ctx.eval_macro_args(*scope_id, id, self.tree);

                let current_scope = ctx.get_current_scope_id();
                ctx.set_current_scope_id(*scope_id).unwrap();

                let (m_node, _) = self.get_node_item(ctx, *macro_id);

                let kids: Vec<_> = m_node.children().map(|n| n.id()).collect();

                for c in kids {
                    pc = self.size_node(ctx, pc, c, pc_symbol_id, current_scope_id)?;
                }

                ctx.set_current_scope_id(current_scope).unwrap();
            }

            ScopeId(scope_id) => {
                ctx.set_current_scope_id(*scope_id).unwrap();
            }

            GrabMem => {
                let args = ctx.ctx.eval_n_args(node, 2,current_scope_id)?;
                let size = args[1];
                pc += size as usize;
            }

            Org => {
                let (value, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                pc = value as usize;
                ctx.add_fixup(id, Item::SetPc(pc));
            }

            SetPc(val) => pc = *val,

            Put => {
                let (value, _) = ctx.ctx.eval_first_arg(node,current_scope_id)?;
                let offset = (value - pc as i64) as isize;
                ctx.add_fixup(id, Item::SetPutOffset(offset));
            }

            Rmb => {
                let (bytes, _) = ctx.ctx.eval_first_arg(node,current_scope_id)?;

                if bytes < 0 {
                    return Err(ctx
                        .ctx
                        .user_error("Argument for RMB must be positive", node, true)
                        .into());
                };

                ctx.add_fixup(id, Item::Skip(bytes as usize));

                pc += bytes as usize;
            }

            Cpu(OpCode(text, ins, amode)) => {
                match amode {
                    AddrModeParseType::Extended(false) => {
                        // If there is a direct page set AND
                        // I can evaluate the arg AND
                        // the instruction supports DIRECT addressing (phew)
                        // I can changing this to a direct page mode instruction
                        // !!!! and it wasn't forced (need someway to propogate this from parse)

                        let mut size = ins.size;

                        let dp_info = get_opcode_info(ins)
                            .and_then(|i_type| i_type.get_instruction(&AddrModeEnum::Direct))
                            .and_then(|ins| ctx.ctx.asm_out.direct_page.map(|dp| (ins, dp)));

                        if let Some((new_ins, dp)) = dp_info {
                            if let Ok((value, _)) = ctx.ctx.eval_first_arg(node, current_scope_id) {
                                let top_byte = ((value >> 8) & 0xff) as u8;

                                if top_byte == dp {
                                    // Here we go!
                                    let new_ins = new_ins.clone();
                                    size = new_ins.size;
                                    let new_item = Cpu(OpCode(
                                        text.clone(),
                                        Box::new(new_ins),
                                        AddrModeParseType::Direct,
                                    ));
                                    ctx.add_fixup(id, new_item);
                                }
                            }
                        }

                        pc += size;
                    }

                    AddrModeParseType::Indexed(..) => {
                        pc = self.size_indexed(ctx, pc, id, current_scope_id)?;
                    }

                    _ => {
                        pc += ins.size;
                    }
                };
            }

            AssignmentFromPc(LabelDefinition::Scoped(symbol_id)) => {
                // TODO: should two types of item rather than this
                // conditional
                let pcv = if node.first_child().is_some() {
                    ctx.set_symbol_value(pc_symbol_id, pc)
                        .expect("Can't set PC symbol value");
                    ctx.ctx.eval_first_arg(node, current_scope_id)?.0
                } else {
                    // Otherwise it's just the current PC
                    pc as i64
                };
                ctx.set_symbol_value(*symbol_id, pcv as usize).unwrap();
            }

            TokenizedFile(..) => {
                for c in ctx.ctx.get_children(node) {
                    pc = self.size_node(ctx, pc, c, pc_symbol_id, current_scope_id)?;
                }
            }

            Fdb(num_of_words) => {
                pc += *num_of_words * 2;
            }

            Fcb(num_of_bytes) => {
                pc += *num_of_bytes;
            }

            Fcc(text) => {
                pc += text.as_bytes().len();
            }

            Zmb => {
                let (v, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                pc += v as usize;
            }

            Zmd => {
                let (v, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                pc += (v * 2) as usize;
            }

            Fill => {
                let (_, c) = ctx.ctx.eval_two_args(node, current_scope_id)?;
                assert!(c >= 0);
                pc += c as usize;
            }

            Cpu(SetDp) => {
                let (dp, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                ctx.set_dp(dp);
            }

            IncBin(file_name) => {
                let r = Self::get_binary_extents(ctx, file_name, node,current_scope_id)?;
                let new_item = IncBinResolved {
                    file: file_name.clone(),
                    r: r.clone(),
                };

                ctx.add_fixup(id, new_item);
                pc += r.len();
            }

            PostFixExpr | WriteBin(..) | IncBinRef(..) | Assignment(..) | Comment(..)
            | StructDef(..) | MacroDef(..) | MacroCall(..) => (),

            _ => {
                let msg = format!("Unable to size {i:?}");
                return Err(ctx.ctx.user_error(msg, node, true).into());
            }
        };

        Ok(pc)
    }

    fn get_binary_extents<P: AsRef<Path>>(
        ctx: &mut AsmCtx,
        file_name: P,
        node: AstNodeRef,
        current_scope_id: u64
    ) -> GResult<std::ops::Range<usize>> {
        let data_len = ctx.get_file_size(file_name.as_ref())?;

        let mut r = 0..data_len;

        let mut c = node.children();

        let offset_size = c
            .next()
            .and_then(|offset| c.next().map(|size| (offset, size)));

        if let Some((offset, size)) = offset_size {
            let offset = ctx.ctx.eval_node(offset,current_scope_id)?;
            let size = ctx.ctx.eval_node(size,current_scope_id)?;
            let offset_usize = offset as usize;
            let size_usize = size as usize;
            let last = (offset_usize + size_usize) - 1;

            if !(r.contains(&offset_usize) && r.contains(&last)) {
                let msg =
                    format!("Trying to grab {offset:04X} {size:04X} from file size {data_len:X}");
                return Err(ctx.ctx.user_error(msg, node, true).into());
            };

            r.start = offset_usize;
            r.end = offset_usize + size_usize;
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
