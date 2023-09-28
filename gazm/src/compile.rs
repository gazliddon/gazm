/// Take the sized AST and compile it into binary
use std::path::Path;

use crate::{
    asmctx::AsmCtx,
    ast::{AstNodeId, AstNodeRef, AstTree},
    binary::BinaryError,
    debug_mess,
    error::{GResult, GazmErrorKind, UserError},
    gazm::ScopeTracker,
    info_mess,
    item::{self, Item},
    item6809::{
        self, AddrModeParseType, IndexParseType,
        MC6809::{self, OpCode, SetDp},
    },
    regutils::*,
    status_mess, gazmsymbols::SymbolScopeId,
};

use emu6809::isa;
use grl_sources::ItemType;

pub fn compile(ctx: &mut AsmCtx, tree: &AstTree) -> GResult<()> {
    let mut writer = ctx.ctx.get_symbols_mut().get_root_writer();

    let pc_symbol_id = writer
        .create_and_set_symbol("*", 0)
        .expect("Can't add symbol for pc");

    let root_id = ctx.ctx.get_symbols().get_root_scope_id();
    let mut compiler = Compiler::new(tree, root_id, pc_symbol_id)?;
    compiler.compile_root(ctx)?;

    let mut writer = ctx.ctx.get_symbols_mut().get_root_writer();
    writer.remove_symbol("*").expect("Can't remove pc symbol");

    Ok(())
}

struct Compiler<'a> {
    tree: &'a AstTree,
    scopes: ScopeTracker,
    pc_symbol_id : SymbolScopeId,
}

impl<'a> Compiler<'a> {
    fn get_node_item(&self, ctx: &AsmCtx, id: AstNodeId) -> (AstNodeId, Item) {
        let (node, i) = self.get_node_item_ref(ctx, id);
        (node.id(), i)
    }

    fn get_node_item_ref(&self, ctx: &AsmCtx, id: AstNodeId) -> (AstNodeRef, Item) {
        let node = self.tree.get(id).unwrap();
        let this_i = &node.value().item;
        let i = ctx.get_fixup_or_default(id, this_i, self.scopes.scope());
        (node, i)
    }

    // fn get_item(&self, ctx: &AsmCtx, id: AstNodeId) -> Item {
    //     let i = &self.get_node(id).value().item;
    //     ctx.get_fixup_or_default(id, i, self.scopes.scope())
    // }

    fn get_node(&self, id: AstNodeId) -> AstNodeRef {
        let node = self.tree.get(id).unwrap();
        node
    }

    pub fn new(tree: &'a AstTree, current_scope_id: u64, pc_symbol_id: SymbolScopeId) -> GResult<Self> {
        let ret = Self {
            tree,
            scopes: ScopeTracker::new(current_scope_id),
            pc_symbol_id,
        };
        Ok(ret)
    }

    fn binary_error(
        &self,
        ctx: &AsmCtx,
        id: AstNodeId,
        e: crate::binary::BinaryError,
    ) -> GazmErrorKind {
        let n = self.get_node(id);
        let info = &ctx.ctx.get_source_info(&n.value().pos).unwrap();
        let msg = e.to_string();
        UserError::from_text(msg, info, true).into()
    }
    fn binary_error_map<T>(
        &self,
        ctx: &AsmCtx,
        id: AstNodeId,
        e: Result<T, BinaryError>,
    ) -> Result<T, GazmErrorKind> {
        e.map_err(|e| self.binary_error(ctx, id, e))
    }

    fn relative_error(&self, ctx: &AsmCtx, id: AstNodeId, val: i64, bits: usize) -> GazmErrorKind {
        let n = self.get_node(id);
        let p = 1 << (bits - 1);

        let message = if val < 0 {
            format!("Branch out of range by {} bytes ({val})", (p + val).abs())
        } else {
            format!("Branch out of range by {} bytes ({val})", val - (p - 1))
        };

        let info = &ctx.ctx.get_source_info(&n.value().pos).unwrap();
        let msg = message;
        UserError::from_text(msg, info, true).into()
    }

    fn compile_indexed(
        &mut self,
        ctx: &mut AsmCtx,
        id: AstNodeId,
        imode: IndexParseType,
        indirect: bool,
    ) -> GResult<()> {
        use item6809::IndexParseType::*;

        let idx_byte = imode.get_index_byte(indirect);
        ctx.binary_mut().write_byte(idx_byte)?;

        let node = self.get_node(id);

        match imode {
            PCOffset | ConstantOffset(..) => {
                panic!("Should not happen")
            }

            ExtendedIndirect => {
                let (val, _) = ctx.ctx.eval_first_arg(node, self.scopes.scope())?;

                let res = ctx.binary_mut().write_uword_check_size(val);
                self.binary_error_map(ctx, id, res)?;
            }

            ConstantWordOffset(_, val) | PcOffsetWord(val) => {
                let res = ctx.binary_mut().write_iword_check_size(val as i64);
                self.binary_error_map(ctx, id, res)?;
            }

            ConstantByteOffset(_, val) | PcOffsetByte(val) => {
                let res = ctx.binary_mut().write_ibyte_check_size(val as i64);
                self.binary_error_map(ctx, id, res)?;
            }
            _ => (),
        }

        Ok(())
    }

    /// Adds a mapping of this source file fragment to a physicl and logical range of memory
    /// ( physical range, logical_range )
    fn add_mapping(
        &self,
        ctx: &mut AsmCtx,
        phys_range: std::ops::Range<usize>,
        range: std::ops::Range<usize>,
        id: AstNodeId,
        i: ItemType,
    ) {
        let pos = self.get_node(id).value().pos.clone();
        ctx.ctx
            .asm_out
            .source_map
            .add_mapping(phys_range, range, &pos, i);
    }

    /// Grab memory and copy it the PC
    fn grab_mem(&self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        let node = self.get_node(id);
        let args = ctx.ctx.eval_n_args(node, 2, self.scopes.scope())?;
        let source = args[0];
        let size = args[1];

        let bytes = ctx.binary().get_bytes(source as usize, size as usize);

        let bytes = self.binary_error_map(ctx, id, bytes)?.to_vec();

        ctx.binary_mut()
            .write_bytes(&bytes)
            .map_err(|e| self.binary_error(ctx, id, e))?;

        Ok(())
    }

    /// Add a binary to write
    fn add_binary_to_write<P: AsRef<Path>>(
        &self,
        ctx: &mut AsmCtx,
        id: AstNodeId,
        path: P,
    ) -> GResult<()> {
        let current_scope_id = self.scopes.scope();

        let node = self.get_node(id);
        let (physical_address, count) = ctx.ctx.eval_two_args(node, current_scope_id)?;

        ctx.add_bin_to_write(
            &path,
            physical_address as usize..(physical_address + count) as usize,
        )?;

        Ok(())
    }

    fn inc_bin_ref<P: AsRef<Path>>(&self, ctx: &mut AsmCtx, file_name: P) -> GResult<()> {
        use crate::binary::BinRef;

        let file = file_name.as_ref().to_path_buf();

        let (.., data) = ctx.read_binary(&file_name)?;

        let dest = ctx.binary().get_write_location().physical;

        let bin_ref = BinRef {
            file: file.clone(),
            start: 0,
            size: data.len(),
            dest,
        };

        ctx.binary_mut().bin_reference(&bin_ref, &data);

        info_mess!(
            "Adding binary reference {} for {:05X} - {:05X}",
            file.to_string_lossy(),
            dest,
            dest + data.len()
        );

        Ok(())
    }

    /// Compile an opcode
    fn compile_opcode(
        &mut self,
        ctx: &mut AsmCtx,
        id: AstNodeId,
        ins: &isa::Instruction,
        amode: AddrModeParseType,
    ) -> GResult<()> {
        use isa::AddrModeEnum::*;

        let node = self.get_node(id);
        let pc = ctx.binary().get_write_address();
        let ins_amode = ins.addr_mode;
        let current_scope_id = self.scopes.scope();

        let res = if ins.opcode > 0xff {
            ctx.binary_mut().write_word(ins.opcode as u16)
        } else {
            ctx.binary_mut().write_byte(ins.opcode as u8)
        };

        self.binary_error_map(ctx, id, res)?;

        match ins_amode {
            Indexed => {
                if let AddrModeParseType::Indexed(imode, indirect) = amode {
                    self.compile_indexed(ctx, id, imode, indirect)?;
                }
            }

            Immediate8 => {
                let (arg, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                let res = ctx.binary_mut().write_byte_check_size(arg);
                self.binary_error_map(ctx, id, res)?;
            }

            Direct => {
                let (arg, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                let res = ctx.binary_mut().write_byte_check_size(arg & 0xff);
                self.binary_error_map(ctx, id, res)?;
            }

            Extended | Immediate16 => {
                let (arg, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                let res = ctx.binary_mut().write_word_check_size(arg);
                self.binary_error_map(ctx, id, res)?;
            }

            Relative => {
                use crate::binary::BinaryError::*;
                let (arg, arg_id) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                let arg_n = self.get_node(arg_id);
                let val = arg - (pc as i64 + ins.size as i64);
                // offset is from PC after Instruction and operand has been fetched
                let res = ctx
                    .ctx
                    .asm_out
                    .binary
                    .write_ibyte_check_size(val)
                    .map_err(|x| match x {
                        DoesNotFit { .. } => self.relative_error(ctx, id, val, 8),
                        DoesNotMatchReference { .. } => self.binary_error(ctx, id, x),
                        _ => ctx.ctx.user_error(format!("{x:?}"), arg_n, false).into(),
                    });

                match &res {
                    Ok(_) => (),
                    Err(_) => {
                        if ctx.ctx.opts.ignore_relative_offset_errors {
                            // messages::warning("Skipping writing relative offset");
                            let res = ctx.binary_mut().write_ibyte_check_size(0);
                            self.binary_error_map(ctx, id, res)?;
                        } else {
                            res?;
                        }
                    }
                }
            }

            Relative16 => {
                use crate::binary::BinaryError::*;

                let (arg, arg_id) = ctx.ctx.eval_first_arg(node, current_scope_id)?;

                let arg_n = self.get_node(arg_id);

                let val = (arg - (pc as i64 + ins.size as i64)) & 0xffff;
                // offset is from PC after Instruction and operand has been fetched
                let res = ctx.binary_mut().write_word_check_size(val);

                res.map_err(|x| match x {
                    DoesNotFit { .. } => self.relative_error(ctx, id, val, 16),
                    DoesNotMatchReference { .. } => self.binary_error(ctx, id, x),
                    _ => ctx.ctx.user_error(format!("{x:?}"), arg_n, true).into(),
                })?;
            }

            Inherent => {}

            RegisterPair => {
                if let AddrModeParseType::RegisterPair(a, b) = amode {
                    let res = ctx.ctx.asm_out.binary.write_byte(reg_pair_to_flags(a, b));
                    self.binary_error_map(ctx, id, res)?;
                } else {
                    panic!("Whut!")
                }
            }

            RegisterSet => {
                let rset = &node.first_child().unwrap().value().item;
                if let Item::Cpu(MC6809::RegisterSet(regs)) = rset {
                    let flags = registers_to_flags(regs);
                    let res = ctx.binary_mut().write_byte(flags);
                    self.binary_error_map(ctx, id, res)?;
                } else {
                    panic!("Whut!")
                }
            }
        };

        // Add memory to source code mapping for this opcode
        let (phys_range, range) = ctx.binary().range_to_write_address(pc);
        self.add_mapping(ctx, phys_range, range, id, ItemType::OpCode);
        Ok(())
    }

    fn incbin_resolved<P: AsRef<Path>>(
        &self,
        ctx: &mut AsmCtx,
        id: AstNodeId,
        file: P,
        r: &std::ops::Range<usize>,
    ) -> GResult<()> {
        status_mess!(
            "Including Binary {} :  offset: {:04X} len: {:04X}",
            file.as_ref().to_string_lossy(),
            r.start,
            r.len()
        );

        let (.., bin) = ctx.read_binary_chunk(file, r.clone())?;

        for val in bin {
            ctx.ctx
                .asm_out
                .binary
                .write_byte(val)
                .map_err(|e| self.binary_error(ctx, id, e))?;
        }
        Ok(())
    }

    fn compile_children(&mut self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        let node = self.get_node(id);
        let kids: Vec<_> = node.children().map(|n| n.id()).collect();
        for c in kids {
            self.compile_node(ctx, c)?;
        }
        Ok(())
    }

    fn compile_root(&mut self, ctx: &mut AsmCtx) -> GResult<()> {
        let scope_id = ctx.ctx.get_symbols().get_root_scope_id();
        self.scopes.set_scope(scope_id);
        self.compile_node(ctx, self.tree.root().id())
    }

    fn compile_node(&mut self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        use item::Item::*;

        let (node_id, i) = self.get_node_item(ctx, id);
        let mut pc = ctx.binary().get_write_address();
        let mut do_source_mapping = ctx.ctx.opts.lst_file.is_some();
        let current_scope_id = self.scopes.scope();

        ctx.set_symbol_value(self.pc_symbol_id, pc)
            .expect("Can't set PC symbol value");

        match i {
            ScopeId(scope_id) => self.scopes.set_scope(scope_id),

            GrabMem => self.grab_mem(ctx, id)?,

            WriteBin(file_name) => self.add_binary_to_write(ctx, id, &file_name)?,

            IncBinRef(file_name) => {
                self.inc_bin_ref(ctx, &file_name)?;
            }

            IncBinResolved { file, r } => {
                self.incbin_resolved(ctx, id, &file, &r)?;
            }

            Skip(skip) => {
                ctx.binary_mut().skip(skip);
            }

            SetPc(new_pc) => {
                ctx.binary_mut().set_write_address(new_pc, 0);

                pc = new_pc;
                debug_mess!("Set PC to {:02X}", pc);
            }

            SetPutOffset(offset) => {
                debug_mess!("Set put offset to {}", offset);
                ctx.binary_mut().set_write_offset(offset);
            }

            Cpu(OpCode(_, ins, amode)) => {
                self.compile_opcode(ctx, id, &ins, amode)?;
            }

            MacroCallProcessed {
                scope_id, macro_id, ..
            } => {
                let node = self.get_node(node_id);
                do_source_mapping = false;
                let ret = ctx.ctx.eval_macro_args(scope_id, node);

                if !ret {
                    let pos = &node.value().pos;
                    let si = ctx.ctx.get_source_info(pos).unwrap();
                    return Err(UserError::from_text(
                        "Couldn't evaluate all macro args",
                        &si,
                        true,
                    )
                    .into());
                }

                self.scopes.push(scope_id);

                {
                    let m_node = self.get_node(macro_id);
                    let kids: Vec<_> = m_node.children().map(|n| n.id()).collect();

                    for c_node in kids {
                        self.compile_node(ctx, c_node)?;
                    }
                }

                self.scopes.pop();
            }

            TokenizedFile(..) => {
                self.compile_children(ctx, id)?;
            }

            Fdb(..) => {
                let node = self.get_node(node_id);

                for n in node.children() {
                    let x = ctx.ctx.eval_node(n, current_scope_id)?;
                    let e = ctx.binary_mut().write_word_check_size(x);
                    self.binary_error_map(ctx, id, e)?;
                }

                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Fcb(..) => {
                let node = self.get_node(node_id);
                for n in node.children() {
                    let x = ctx.ctx.eval_node(n, current_scope_id)?;
                    let e = ctx.binary_mut().write_byte_check_size(x);
                    self.binary_error_map(ctx, id, e)?;
                }
                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Fcc(text) => {
                for c in text.as_bytes() {
                    let e = ctx.binary_mut().write_byte(*c);
                    self.binary_error_map(ctx, id, e)?;
                }
                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Zmb => {
                let node = self.get_node(node_id);
                let (bytes, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                for _ in 0..bytes {
                    let e = ctx.binary_mut().write_byte(0);
                    self.binary_error_map(ctx, id, e)?;
                }
                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Zmd => {
                let node = self.get_node(node_id);
                let (words, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                for _ in 0..words {
                    let e = ctx.binary_mut().write_word(0);
                    self.binary_error_map(ctx, id, e)?;
                }

                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Fill => {
                let node = self.get_node(node_id);
                let (size, byte) = ctx.ctx.eval_two_args(node, current_scope_id)?;

                for _ in 0..size {
                    let e = ctx.binary_mut().write_ubyte_check_size(byte);
                    self.binary_error_map(ctx, id, e)?;
                }

                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Exec => {
                let node = self.get_node(node_id);
                let (exec_addr, _) = ctx.ctx.eval_first_arg(node, current_scope_id)?;
                ctx.set_exec_addr(exec_addr as usize);
            }

            IncBin(..) | Org | AssignmentFromPc(..) | Assignment(..) | Comment(..) | Rmb
            | StructDef(..) | MacroDef(..) | MacroCall(..) | Cpu(SetDp) | Import => (),
            _ => {
                panic!("Can't compile {i:?}");
            }
        }

        if do_source_mapping {
            let node = self.get_node(node_id);
            ctx.add_source_mapping(&node.value().pos, pc);
        }

        Ok(())
    }
}
