use std::collections::{HashMap, HashSet};
use std::os::unix::prelude::AsRawFd;
use std::path::Path;

use crate::{
    asmctx::AsmCtx,
    ast::{AstNodeId, AstNodeRef, AstTree},
    astformat::as_string,
    ctx::Opts,
    error::{GResult, GazmErrorType, UserError},
    fixerupper::FixerUpper,
    item::{self, AddrModeParseType, IndexParseType, Item, Node},
    messages::debug_mess,
    messages::{info, messages},
    regutils::*,
    status_mess,
};

use emu::utils::sources::Position;

use emu::{isa::Instruction, utils::sources::fileloader::FileIo, utils::sources::ItemType};

pub fn compile(ctx: &mut AsmCtx, tree: &AstTree) -> GResult<()> {
    let compiler = Compiler::new(tree)?;
    ctx.set_root_scope();
    compiler.compile_node(ctx, tree.root().id())
}

struct Compiler<'a> {
    tree: &'a AstTree,
}

impl<'a> Compiler<'a> {
    fn get_node_item(&self, ctx: &AsmCtx, id: AstNodeId) -> (AstNodeRef, Item) {
        let node = self.tree.get(id).unwrap();
        let this_i = &node.value().item;
        let i = ctx.get_fixup_or_default(id, this_i);
        (node, i)
    }

    pub fn new(tree: &'a AstTree) -> GResult<Self> {
        let ret = Self { tree };

        Ok(ret)
    }

    fn binary_error(
        &self,
        ctx: &AsmCtx,
        id: AstNodeId,
        e: crate::binary::BinaryError,
    ) -> GazmErrorType {
        let (n, _) = self.get_node_item(ctx, id);
        let info = &ctx.ctx.get_source_info(&n.value().pos).unwrap();
        let msg = e.to_string();
        UserError::from_text(msg, info, true).into()
    }

    fn relative_error(&self, ctx: &AsmCtx, id: AstNodeId, val: i64, bits: usize) -> GazmErrorType {
        let (n, _) = self.get_node_item(ctx, id);
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

    pub fn compile(&self, ctx: &mut AsmCtx) -> GResult<()> {
        ctx.set_root_scope();
        self.compile_node(ctx, self.tree.root().id())
    }

    fn compile_indexed(
        &self,
        ctx: &mut AsmCtx,
        id: AstNodeId,
        imode: IndexParseType,
        indirect: bool,
    ) -> GResult<()> {
        let idx_byte = imode.get_index_byte(indirect);
        let (node, _) = self.get_node_item(ctx, id);

        let si = ctx.ctx.get_source_info(&node.value().pos).unwrap();
        debug_mess!("{} {:?}", si.line_str, imode);

        ctx.binary_mut().write_byte(idx_byte)?;

        use item::IndexParseType::*;

        match imode {
            PCOffset | ConstantOffset(..) => {
                panic!("Should not happen")
            }

            ExtendedIndirect => {
                let (val, _) = ctx.ctx.eval_first_arg(node)?;
                ctx.binary_mut().write_uword_check_size(val)?;
            }

            ConstantWordOffset(_, val) | PcOffsetWord(val) => {
                ctx.binary_mut().write_iword_check_size(val as i64)?;
            }

            ConstantByteOffset(_, val) | PcOffsetByte(val) => {
                ctx.binary_mut().write_ibyte_check_size(val as i64)?;
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
        let pos = self.get_node_item(ctx, id).0.value().pos.clone();
        ctx.ctx
            .asm_out
            .source_map
            .add_mapping(phys_range, range, &pos, i);
    }

    /// Grab memory and copy it the PC
    fn grab_mem(&self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        let (n, _) = self.get_node_item(ctx, id);
        let args = ctx.ctx.eval_n_args(n, 2)?;
        let source = args[0];
        let size = args[1];

        let bytes = ctx
            .binary()
            .get_bytes(source as usize, size as usize)
            .to_vec();

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
        let (node, _) = self.get_node_item(ctx, id);
        let (physical_address, count) = ctx.ctx.eval_two_args(node)?;

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

        messages().info(format!(
            "Adding binary reference {} for {:05X} - {:05X}",
            file.to_string_lossy(),
            dest,
            dest + data.len()
        ));

        Ok(())
    }

    /// Compile an opcode
    fn opcode(
        &self,
        ctx: &mut AsmCtx,
        id: AstNodeId,
        ins: &Instruction,
        amode: &AddrModeParseType,
    ) -> GResult<()> {
        use emu::isa::AddrModeEnum::*;

        let (node, _) = self.get_node_item(ctx, id);

        let x = messages();
        let pc = ctx.binary().get_write_address();

        let ins_amode = ins.addr_mode;

        if ins.opcode > 0xff {
            ctx.binary_mut().write_word(ins.opcode as u16)?;
        } else {
            ctx.binary_mut().write_byte(ins.opcode as u8)?;
        }

        match ins_amode {
            Indexed => {
                if let AddrModeParseType::Indexed(imode, indirect) = amode {
                    self.compile_indexed(ctx, id, *imode, *indirect)?;
                }
            }

            Immediate8 => {
                let (arg, _) = ctx.ctx.eval_first_arg(node)?;
                ctx.binary_mut().write_byte_check_size(arg)?;
            }

            Direct => {
                let (arg, _) = ctx.ctx.eval_first_arg(node)?;
                ctx.binary_mut().write_byte_check_size(arg & 0xff)?;
            }

            Extended | Immediate16 => {
                let (arg, _) = ctx.ctx.eval_first_arg(node)?;
                ctx.binary_mut().write_word_check_size(arg)?;
            }

            Relative => {
                let (arg, arg_id) = ctx.ctx.eval_first_arg(node)?;
                let (arg_n, _) = self.get_node_item(ctx, arg_id);
                let val = arg - (pc as i64 + ins.size as i64);
                // offset is from PC after Instruction and operand has been fetched
                use crate::binary::BinaryError::*;
                let res = ctx
                    .ctx
                    .asm_out
                    .binary
                    .write_ibyte_check_size(val)
                    .map_err(|x| match x {
                        DoesNotFit { .. } => self.relative_error(ctx, id, val, 8),
                        DoesNotMatchReference { .. } => self.binary_error(ctx, id, x),
                        _ => ctx.ctx.user_error(format!("{:?}", x), arg_n, false).into(),
                    });

                match &res {
                    Ok(_) => (),
                    Err(_) => {
                        if ctx.ctx.opts.ignore_relative_offset_errors {
                            // x.warning(e.pretty().unwrap());
                            x.warning("Skipping writing relative offset");
                            ctx.binary_mut().write_ibyte_check_size(0)?;
                        } else {
                            res?;
                        }
                    }
                }
            }

            Relative16 => {
                use crate::binary::BinaryError::*;

                let (arg, arg_id) = ctx.ctx.eval_first_arg(node)?;
                let (arg_n, _) = self.get_node_item(ctx, arg_id);

                let val = (arg - (pc as i64 + ins.size as i64)) & 0xffff;
                // offset is from PC after Instruction and operand has been fetched
                let res = ctx.binary_mut().write_word_check_size(val);

                res.map_err(|x| match x {
                    DoesNotFit { .. } => self.relative_error(ctx, id, val, 16),
                    DoesNotMatchReference { .. } => self.binary_error(ctx, id, x),
                    _ => ctx.ctx.user_error(format!("{:?}", x), arg_n, true).into(),
                })?;
            }

            Inherent => {}

            RegisterPair => {
                if let AddrModeParseType::RegisterPair(a, b) = amode {
                    ctx.ctx
                        .asm_out
                        .binary
                        .write_byte(reg_pair_to_flags(*a, *b))?;
                } else {
                    panic!("Whut!")
                }
            }

            RegisterSet => {
                let rset = &node.first_child().unwrap().value().item;

                if let Item::RegisterSet(regs) = rset {
                    let flags = registers_to_flags(regs);
                    ctx.binary_mut().write_byte(flags)?;
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
        let msg = format!(
            "Including Binary {} :  offset: {:04X} len: {:04X}",
            file.as_ref().to_string_lossy(),
            r.start,
            r.len()
        );

        messages().status(msg);

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

    fn compile_children(&self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        let (node, _) = self.get_node_item(ctx, id);
        for c in node.children() {
            self.compile_node(ctx, c.id())?;
        }
        Ok(())
    }

    fn compile_node(&self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        use item::Item::*;

        let (node, i) = self.get_node_item(ctx, id);

        let mut pc = ctx.binary().get_write_address();
        let pos = node.value().pos.clone();

        let mut do_source_mapping = ctx.ctx.opts.lst_file.is_some();

        match i {
            Scope(opt) => {
                ctx.set_root_scope();
                if opt != "root" {
                    ctx.set_scope(&opt);
                }
            }

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

            OpCode(ins, amode) => {
                self.opcode(ctx, id, &ins, &amode)?;
            }

            MacroCallProcessed { scope_id, macro_id, .. } => {
                do_source_mapping = false;
                let (m_node, _) = self.get_node_item(ctx, macro_id);
                let ret = ctx.ctx.eval_macro_args(scope_id, node);

                if !ret {
                    let si = ctx.ctx.get_source_info(&pos).unwrap();
                    return Err(UserError::from_text(
                        "Couldn't evaluate all macro args",
                        &si,
                        true,
                    )
                    .into());
                }

                let prev_scop = ctx.get_current_scope_id();
                ctx.set_scope_from_id(scope_id).unwrap();

                for c_node in m_node.children() {
                    self.compile_node(ctx, c_node.id())?;
                }

                ctx.set_scope_from_id(prev_scop).unwrap();
            }

            Block | TokenizedFile(..) => {
                self.compile_children(ctx, id)?;
            }

            Fdb(..) => {
                for n in node.children() {
                    let x = ctx.ctx.eval_node(n)?;
                    ctx.binary_mut().write_word_check_size(x)?;
                }

                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Fcb(..) => {
                for n in node.children() {
                    let x = ctx.ctx.eval_node(n)?;
                    ctx.binary_mut().write_byte_check_size(x)?;
                }
                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Fcc(text) => {
                for c in text.as_bytes() {
                    ctx.binary_mut().write_byte(*c)?;
                }
                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Zmb => {
                let (bytes, _) = ctx.ctx.eval_first_arg(node)?;
                for _ in 0..bytes {
                    ctx.binary_mut().write_byte(0)?;
                }
                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Zmd => {
                let (words, _) = ctx.ctx.eval_first_arg(node)?;
                for _ in 0..words {
                    ctx.binary_mut().write_word(0)?;
                }

                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Fill => {
                let (byte, size) = ctx.ctx.eval_two_args(node)?;

                for _ in 0..size {
                    ctx.binary_mut().write_ubyte_check_size(byte)?;
                }

                let (phys_range, range) = ctx.binary().range_to_write_address(pc);
                self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
            }

            Exec => {
                let (exec_addr, _) = ctx.ctx.eval_first_arg(node)?;
                ctx.set_exec_addr(exec_addr as usize);
            }

            IncBin(..) | Org | AssignmentFromPc(..) | Assignment(..) | Comment(..) | Rmb
            | StructDef(..) | MacroDef(..) | MacroCall(..) | SetDp => (),
            _ => {
                panic!("Can't compile {:?}", i);
            }
        }

        if do_source_mapping {
            ctx.add_source_mapping(&node.value().pos, pc);
        }

        Ok(())
    }
}
