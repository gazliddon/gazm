#![forbid(unused_imports)]
use super::{
    bytesizes::{ByteSize, ByteSizes},
    Assembler,
    scopetracker::ScopeTracker,
};
/// Take the AST and work out the sizes of everything
/// Resolve labels where we can
use crate::{
    ast::{Ast, AstNodeId, AstNodeRef},
    error::GResult,
    // parse::util::{ByteSize, ByteSizes},
    // parse6809::opcodes::get_opcode_info,
    frontend::get_opcode_info,
    item::{self, Item, LabelDefinition},
    item6809::{
        AddrModeParseType,
        MC6809::{OpCode, SetDp},
    },
};

use emu6809::isa::AddrModeEnum;
use std::path::Path;

/// Ast tree sizer
/// gets the size of everything
/// assigns values to labels that
/// are defined by value of PC
struct Sizer<'a> {
    tree: &'a Ast,
    scopes: ScopeTracker,
    pc: usize,
    // pc_symbol_id: SymbolScopeId,
}

pub fn size(asm: &mut Assembler, ast_tree: &Ast) -> GResult<()> {
    let _ = Sizer::try_new(ast_tree, asm)?;
    Ok(())
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

    pub fn try_new(tree: &'a Ast, asm: &mut Assembler) -> GResult<Sizer<'a>> {
        let pc = 0;

        asm.set_pc_symbol(pc).expect("Can't set PC symbol");

        let root_id = asm.get_symbols().get_root_scope_id();

        let mut ret = Self {
            tree,
            scopes: ScopeTracker::new(root_id),
            pc,
        };

        let id = ret.tree.as_ref().root().id();
        ret.size_node(asm, id)?;

        Ok(ret)
    }

    fn size_indexed(&mut self, asm: &mut Assembler, id: AstNodeId) -> GResult<()> {
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
                    let (v, _) = asm.eval_first_arg(node, current_scope_id)?;

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

                    asm.add_fixup(id, new_item, current_scope_id);
                }

                PCOffset => {
                    let node = self.get_node(id);
                    let (v, _) = asm.eval_first_arg(node, current_scope_id)?;
                    self.advance_pc(1);

                    let new_amode = match v.byte_size() {
                        ByteSizes::Zero => PcOffsetByte(0),
                        ByteSizes::Bits5(v) | ByteSizes::Byte(v) => PcOffsetByte(v),
                        ByteSizes::Word(v) => {
                            self.advance_pc(1);
                            PcOffsetWord(v)
                        }
                    };

                    let new_item =
                        OpCode(text, ins, AddrModeParseType::Indexed(new_amode, indirect));
                    asm.add_fixup(id, new_item, current_scope_id);
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

    fn size_node(&mut self, asm: &mut Assembler, id: AstNodeId) -> GResult<()> {
        use item::Item::*;

        let node = self.get_node(id);
        let i = &node.value().item.clone();
        let current_scope_id = self.scopes.scope();

        asm.set_pc_symbol(self.get_pc())
            .expect("Can't set PC symbol value");

        match &i {
            MacroCallProcessed {
                scope_id, macro_id, ..
            } => {
                asm.eval_macro_args_node(*scope_id, id, self.tree);

                self.scopes.push(*scope_id);

                let m_node = self.get_node(*macro_id);
                let kids: Vec<_> = m_node.children().map(|n| n.id()).collect();
                for c in kids {
                    self.size_node(asm, c)?;
                }

                self.scopes.pop();
            }

            ScopeId(scope_id) => self.scopes.set_scope(*scope_id),

            GrabMem => {
                let args = asm.eval_n_args(node, 2, current_scope_id)?;
                let size = args[1];
                self.advance_pc(size as usize);
            }

            Org => {
                let pc = asm.eval_first_arg(node, current_scope_id)?.0 as usize;
                asm.add_fixup(id, Item::SetPc(pc), current_scope_id);
                self.set_pc(pc);
            }

            SetPc(val) => {
                self.set_pc(*val);
            }

            Put => {
                let (value, _) = asm.eval_first_arg(node, current_scope_id)?;
                let offset = (value - self.get_pc() as i64) as isize;
                asm.add_fixup(id, Item::SetPutOffset(offset), current_scope_id);
            }

            Rmb => {
                let (bytes, _) = asm.eval_first_arg(node, current_scope_id)?;

                if bytes < 0 {
                    return Err(asm
                        .user_error("Argument for RMB must be positive", node, true)
                        .into());
                };

                asm.add_fixup(id, Item::Skip(bytes as usize), current_scope_id);
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
                            .and_then(|ins| asm.asm_out.direct_page.map(|dp| (ins, dp)));

                        if let Some((new_ins, dp)) = dp_info {
                            if let Ok((value, _)) = asm.eval_first_arg(node, current_scope_id) {
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
                                    asm.add_fixup(id, new_item, current_scope_id);
                                }
                            }
                        }
                        self.advance_pc(size);
                    }

                    AddrModeParseType::Indexed(..) => {
                        self.size_indexed(asm, id)?;
                    }

                    _ => {
                        self.advance_pc(ins.size);
                    }
                };
            }

            AssignmentFromPc(LabelDefinition::Scoped(symbol_id)) => {
                let pcv = if node.first_child().is_some() {
                    // If we have an arg then evaluate the arg
                    asm.eval_first_arg(node, current_scope_id)?.0
                } else {
                    // Otherwise it's just the current PC
                    self.get_pc() as i64
                };

                asm.set_symbol_value(*symbol_id, pcv as usize).unwrap();
            }

            TokenizedFile(..) => {
                for c in asm.get_children(node) {
                    self.size_node(asm, c)?;
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
                let (v, _) = asm.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                self.advance_pc(v as usize)
            }

            Zmd => {
                let (v, _) = asm.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                self.advance_pc((v * 2) as usize)
            }

            Fill => {
                let (size, _val) = asm.eval_two_args(node, current_scope_id)?;
                assert!(size >= 0);
                self.advance_pc(size as usize);
            }

            Cpu(SetDp) => {
                let (dp, _) = asm.eval_first_arg(node, current_scope_id)?;
                asm.set_dp(dp);
            }

            IncBin(file_name) => {
                let r = self.get_binary_extents(asm, file_name, node)?;
                let new_item = IncBinResolved {
                    file: file_name.clone(),
                    r: r.clone(),
                };

                asm.add_fixup(id, new_item, current_scope_id);
                self.advance_pc(r.len())
            }

            PostFixExpr | WriteBin(..) | IncBinRef(..) | Assignment(..) | Comment(..)
            | StructDef(..) | MacroDef(..) | MacroCall(..) | Import => (),

            _ => {
                let msg = format!("Unable to size {i:?}");
                return Err(asm.user_error(msg, node, true).into());
            }
        };

        Ok(())
    }

    fn get_binary_extents<P: AsRef<Path>>(
        &self,
        asm: &Assembler,
        file_name: P,
        node: AstNodeRef,
    ) -> GResult<std::ops::Range<usize>> {
        use itertools::Itertools;

        let data_len = asm.get_file_size(&file_name)?;

        let mut r = 0..data_len;

        let current_scope_id = self.scopes.scope();

        if let Some((offset, size)) = node.children().collect_tuple() {
            let offset = asm.eval_node(offset, current_scope_id)?;
            let size = asm.eval_node(size, current_scope_id)?;
            let offset_usize = offset as usize;
            let size_usize = size as usize;
            let last = (offset_usize + size_usize) - 1;

            if !(r.contains(&offset_usize) && r.contains(&last)) {
                let msg =
                    format!("Trying to grab {offset:04X} {size:04X} from file size {data_len:X}");
                return Err(asm.user_error(msg, node, true).into());
            };

            r.start = offset_usize;
            r.end = offset_usize + size_usize;
        } else {
            panic!("Should not happen!")
        }

        Ok(r)
    }

    fn get_node(&self, id: AstNodeId) -> AstNodeRef {
        self.tree.as_ref().get(id).expect("Can't fetch node")
    }
}
