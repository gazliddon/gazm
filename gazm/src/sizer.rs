/// Take the AST and work out the sizes of everything
/// Resolve labels where we can
use crate::{
    asmctx::AsmCtx,
    ast::{AstNodeId, AstNodeRef, AstTree},
    error::GResult,
    gazm::ScopeTracker,
    info_mess,
    item::{self, Item, LabelDefinition},
    item6809::{
        AddrModeParseType,
        MC6809::{OpCode, SetDp},
    },
    parse::util::{ByteSize, ByteSizes},
    parse6809::opcodes::get_opcode_info,
    gazmsymbols::SymbolScopeId,
};

use emu6809::isa::AddrModeEnum;

use std::path::Path;

/// Ast tree sizer
/// gets the size of everything
/// assigns values to labels that
/// are defined by value of PC
struct Sizer<'a> {
    tree: &'a AstTree,
    scopes: ScopeTracker,
    pc: usize,
    pc_symbol_id: SymbolScopeId,
}

pub fn size_tree(ctx: &mut AsmCtx, tree: &AstTree) -> GResult<()> {
    crate::messages::info("Sizing tree", |_x| {
        let _sizer = Sizer::try_new(tree, ctx)?;
        info_mess!("done");
        Ok(())
    })
}

impl<'a> Sizer<'a> {
    fn advance_pc(&mut self, val: usize) {
        assert!(self.pc < 65536);
        self.pc += val;
    }

    fn get_pc(&self) -> usize {
        self.pc
    }

    fn set_pc(&mut self, val: usize) {
        self.pc = val;
        assert!(self.pc < 65536);
    }

    pub fn try_new(tree: &'a AstTree, ctx: &mut AsmCtx) -> GResult<Sizer<'a>> {
        let pc = 0;
        let mut writer = ctx.ctx.get_symbols_mut().get_root_writer();

        let pc_symbol_id = writer
            .create_and_set_symbol("*", pc)
            .expect("Can't add symbol for pc");

        let root_id = ctx.ctx.get_symbols().get_root_scope_id();

        let mut ret = Self {
            tree,
            scopes: ScopeTracker::new(root_id),
            pc: 0,
            pc_symbol_id,
        };

        let id = ret.tree.root().id();
        ret.size_node(ctx, id)?;

        let mut writer = ctx.ctx.get_symbols_mut().get_root_writer();
        writer.remove_symbol("*").expect("Can't remove pc symbol");

        Ok(ret)
    }

    fn size_indexed(&mut self, ctx_mut: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        use Item::*;
        // let i = &self.get_node(id).value().item;

        if let Cpu(OpCode(text, ins, AddrModeParseType::Indexed(pmode, indirect))) =
            &self.get_node(id).value().item
        {
            let current_scope_id = self.scopes.scope();
            let pmode = *pmode;
            let indirect = *indirect;
            let text = text.clone();
            let ins = ins.clone();

            self.advance_pc(ins.size);
            use crate::item6809::IndexParseType::*;

            match pmode {
                Zero(..) | AddA(..) | AddB(..) | AddD(..) | Plus(..) | PlusPlus(..) | Sub(..)
                | SubSub(..) => (),

                ConstantByteOffset(..)
                | PcOffsetByte(..)
                | PcOffsetWord(..)
                | ConstantWordOffset(..)
                | Constant5BitOffset(..) => {}

                ConstantOffset(r) => {
                    let node = self.get_node(id);
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
                            self.advance_pc(2);
                            ConstantWordOffset(r, v)
                        }
                        ByteSizes::Byte(v) => {
                            self.advance_pc(1);
                            ConstantByteOffset(r, v)
                        }
                    };

                    let new_item =
                        OpCode(text, ins, AddrModeParseType::Indexed(new_amode, indirect));

                    ctx_mut.add_fixup(id, new_item, current_scope_id);
                }

                PCOffset => {
                    let node = self.get_node(id);
                    let (v, _) = ctx_mut.ctx.eval_first_arg(node, current_scope_id)?;
                    self.advance_pc(1);

                    let new_amode = match v.byte_size() {
                        ByteSizes::Zero => PcOffsetByte(0),
                        ByteSizes::Bits5(v) | ByteSizes::Byte(v) => PcOffsetByte(v),
                        ByteSizes::Word(v) => {
                            self.advance_pc(1);
                            PcOffsetWord(v)
                        }
                    };

                    let new_item = OpCode(
                        text,
                        ins,
                        AddrModeParseType::Indexed(new_amode, indirect),
                    );
                    ctx_mut.add_fixup(id, new_item, current_scope_id);
                }

                ExtendedIndirect => {
                    self.advance_pc(2);
                }
            };
            Ok(())
        } else {
            panic!()
        }
    }

    fn size_node(&mut self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        use item::Item::*;

        let node = self.get_node(id);
        let i = &node.value().item.clone();
        let current_scope_id = self.scopes.scope();

        ctx.set_symbol_value(self.pc_symbol_id, self.get_pc())
            .expect("Can't set PC symbol value");

        match &i {
            MacroCallProcessed {
                scope_id, macro_id, ..
            } => {
                ctx.eval_macro_args(*scope_id, id, self.tree);

                self.scopes.push(*scope_id);

                let m_node = self.get_node(*macro_id);
                let kids: Vec<_> = m_node.children().map(|n| n.id()).collect();
                for c in kids {
                    self.size_node(ctx, c)?;
                }

                self.scopes.pop();
            }

            ScopeId(scope_id) => self.scopes.set_scope(*scope_id),

            GrabMem => {
                let args = ctx.ctx.eval_n_args(node, 2, current_scope_id)?;
                let size = args[1];
                self.advance_pc(size as usize);
            }

            Org => {
                let pc = ctx.ctx.eval_first_arg(node, current_scope_id)?.0 as usize;
                ctx.add_fixup(id, Item::SetPc(pc), current_scope_id);
                self.set_pc(pc);
            }

            SetPc(val) => {
                self.set_pc(*val);
            }

            Put => {
                let (value, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                let offset = (value - self.get_pc() as i64) as isize;
                ctx.add_fixup(id, Item::SetPutOffset(offset), current_scope_id);
            }

            Rmb => {
                let (bytes, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;

                if bytes < 0 {
                    return Err(ctx
                        .ctx
                        .user_error("Argument for RMB must be positive", node, true)
                        .into());
                };

                ctx.add_fixup(id, Item::Skip(bytes as usize), current_scope_id);
                self.advance_pc(bytes as usize);
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
                                    ctx.add_fixup(id, new_item, current_scope_id);
                                }
                            }
                        }
                        self.advance_pc(size);
                    }

                    AddrModeParseType::Indexed(..) => {
                        self.size_indexed(ctx, id)?;
                    }

                    _ => {
                        self.advance_pc(ins.size);
                    }
                };
            }

            AssignmentFromPc(LabelDefinition::Scoped(symbol_id)) => {
                let pcv = if node.first_child().is_some() {
                    // If we have an arg then evaluate the arg
                    ctx.ctx.eval_first_arg(node, current_scope_id)?.0
                } else {
                    // Otherwise it's just the current PC
                    self.get_pc() as i64
                };

                ctx.set_symbol_value(*symbol_id, pcv as usize).unwrap();
            }

            TokenizedFile(..) => {
                for c in ctx.ctx.get_children(node) {
                    self.size_node(ctx, c)?;
                }
            }

            Fdb(num_of_words) => self.advance_pc(*num_of_words * 2),

            Fcb(num_of_bytes) => {
                self.advance_pc(*num_of_bytes);
            }

            Fcc(text) => {
                self.advance_pc(text.as_bytes().len());
            }

            Zmb => {
                let (v, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                self.advance_pc(v as usize)
            }

            Zmd => {
                let (v, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                self.advance_pc((v * 2) as usize)
            }

            Fill => {
                let (_, c) = ctx.ctx.eval_two_args(node, current_scope_id)?;
                assert!(c >= 0);
                self.advance_pc(c as usize);
            }

            Cpu(SetDp) => {
                let (dp, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                ctx.set_dp(dp);
            }

            IncBin(file_name) => {
                let r = self.get_binary_extents(ctx, file_name, node)?;
                let new_item = IncBinResolved {
                    file: file_name.clone(),
                    r: r.clone(),
                };

                ctx.add_fixup(id, new_item, current_scope_id);
                self.advance_pc(r.len())
            }

            PostFixExpr | WriteBin(..) | IncBinRef(..) | Assignment(..) | Comment(..)
            | StructDef(..) | MacroDef(..) | MacroCall(..) | Import => (),

            _ => {
                let msg = format!("Unable to size {i:?}");
                return Err(ctx.ctx.user_error(msg, node, true).into());
            }
        };

        Ok(())
    }

    fn get_binary_extents<P: AsRef<Path>>(
        &self,
        ctx: &mut AsmCtx,
        file_name: P,
        node: AstNodeRef,
    ) -> GResult<std::ops::Range<usize>> {
        use itertools::Itertools;

        let data_len = ctx.get_file_size(file_name.as_ref())?;

        let mut r = 0..data_len;

        let current_scope_id = self.scopes.scope();

        if let Some((offset, size)) = node.children().collect_tuple() {
            let offset = ctx.ctx.eval_node(offset, current_scope_id)?;
            let size = ctx.ctx.eval_node(size, current_scope_id)?;
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
        } else {
            panic!("Should not happen!")
        }

        Ok(r)
    }

    fn get_node(&self, id: AstNodeId) -> AstNodeRef {
        self.tree.get(id).expect("Can't fetch node")
    }
}
