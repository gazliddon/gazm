use crate::asmctx::AsmCtx;
use crate::fixerupper::FixerUpper;
use std::collections::{HashMap, HashSet};
use utils::sources::ItemType;

use crate::ast::{AstNodeId, AstNodeRef, AstTree};
use crate::item::{self, AddrModeParseType, IndexParseType, Item, Node};
use crate::messages::{info, messages};
use std::path::Path;

use crate::error::UserError;

use emu::isa::Instruction;

use crate::ctx::Opts;
use crate::gasm::{GResult, GasmError};

use crate::regutils::*;

pub struct Compiler<'a> {
    tree: &'a AstTree,
    opts: Opts,
}

impl<'a> Compiler<'a> {
    fn get_node_item(&self, ctx : &AsmCtx, id : AstNodeId) -> (AstNodeRef, Item) {
        let node = self.tree.get(id).unwrap();
        let this_i = &node.value().item;
        let i  = ctx.get_fixup_or_default(id, this_i);
        (node, i)
    }

    pub fn new(ctx: &'a mut AsmCtx, opts: Opts, tree: &'a AstTree) -> GResult<Self> {
        if let Some(file) = &opts.as6809_lst {
            messages().status(format!("Loading map file {}", file));
            let m = crate::as6809::MapFile::new(&file)?;
            ctx.binary.addr_reference(m);
        }

        if let Some(file) = &opts.as6809_sym {
            crate::as6809::add_reference_syms(file, &mut ctx.eval.symbols)?;
        }

        let ret = Self { tree, opts };

        Ok(ret)
    }

    fn binary_error(&self, ctx: &AsmCtx, id: AstNodeId, e: crate::binary::BinaryError) -> GasmError {
        let (n,_) = self.get_node_item(ctx,id);
        let info = &ctx.eval.get_source_info(&n.value().pos).unwrap();
        let msg = e.to_string();
        UserError::from_text(msg, info, true).into()
    }

    fn relative_error(&self, ctx: &AsmCtx, id: AstNodeId, val: i64, bits: usize) -> GasmError {
        let (n,_) = self.get_node_item(ctx,id);
        let p = 1 << (bits - 1);

        let message = if val < 0 {
            format!("Branch out of range by {} bytes ({val})", (p + val).abs())
        } else {
            format!("Branch out of range by {} bytes ({val})", val - (p - 1))
        };

        let info = &ctx.eval.get_source_info(&n.value().pos).unwrap();
        let msg = message;
        UserError::from_text(msg, info, true).into()
    }

    pub fn compile(&self, ctx: &mut AsmCtx) -> GResult<()> {
        self.compile_node(ctx, self.tree.root().id())?;
        Ok(())
    }

    fn compile_indexed(
        &self,
        ctx: &mut AsmCtx,
        id: AstNodeId,
        imode: IndexParseType,
        indirect: bool,
    ) -> GResult<()> {
        let idx_byte = imode.get_index_byte(indirect);
        let (node,_) = self.get_node_item(ctx, id);

        let si = ctx.eval.get_source_info(&node.value().pos).unwrap();

        messages().debug(format!("{} {:?}", si.line_str, imode));

        ctx.binary
            .write_byte(idx_byte)
            .map_err(|e| self.binary_error(ctx, id, e))?;

        use item::IndexParseType::*;

        match imode {
            PCOffset | ConstantOffset(..) => {
                panic!("Should not happen")
            }

            ExtendedIndirect => {
                let (val, _) = ctx.eval.eval_first_arg(node)?;
                ctx.binary
                    .write_uword_check_size(val)
                    .map_err(|e| self.binary_error(ctx, id, e))?;
            }

            ConstantWordOffset(_, val) | PcOffsetWord(val) => {
                ctx.binary
                    .write_iword_check_size(val as i64)
                    .map_err(|e| self.binary_error(ctx, id, e))?;
            }

            ConstantByteOffset(_, val) | PcOffsetByte(val) => {
                ctx.binary
                    .write_ibyte_check_size(val as i64)
                    .map_err(|e| self.binary_error(ctx, id, e))?;
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

        let pos = self.get_node_item(ctx,id).0.value().pos.clone();
        ctx.source_map.add_mapping(phys_range, range, &pos, i);
    }

    /// Grab memory and copy it the PC
    fn grab_mem(&self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        let (n,_) = self.get_node_item(ctx, id);
        let args = ctx.eval.eval_n_args(n, 2)?;
        let source = args[0];
        let size = args[1];

        let bytes = ctx
            .binary
            .get_bytes(source as usize, size as usize)
            .to_vec();

        ctx.binary
            .write_bytes(&bytes)
            .map_err(|e| self.binary_error(ctx, id, e))?;

        Ok(())
    }

    /// Write out a slice of memory
    fn write_bin(&self, ctx: &mut AsmCtx, id: AstNodeId, _file_name: &Path) -> GResult<()> {
        let (node,_) = self.get_node_item(ctx, id);
        let (addr, size) = ctx.eval.eval_two_args(node)?;
        let _mem = ctx.binary.get_bytes(addr as usize, size as usize);
        // let p = .ctx.ctx.write(file_name, mem);
        // messages().info(format!(
        //     "Write mem: {} {addr:05X} {size:05X}",
        //     p.to_string_lossy()
        // ));
        // Ok(())
        panic!()
    }

    fn inc_bin_ref(&self, _ctx: &mut AsmCtx, _id: AstNodeId, _file_name: &Path) -> GResult<()> {
        panic!()
        // let data = self.ctx.get_binary(file_name, id)?;
        // let dest = self.binary.get_write_location().physical;

        // let bin_ref = BinRef {
        //     file: file_name.to_path_buf(),
        //     start: 0,
        //     size: data.len(),
        //     dest,
        // };

        // self.binary.bin_reference(&bin_ref, &data);
        // let file_name = file_name.to_string_lossy();

        // messages().info(format!(
        //     "Adding binary reference {file_name} for {:05X} - {:05X}",
        //     dest,
        //     dest + data.len()
        // ));
        // Ok(())
    }

    fn opcode(
        &self,
        ctx: &mut AsmCtx,
        id: AstNodeId,
        ins: &Instruction,
        amode: &AddrModeParseType,
    ) -> GResult<()> {
        let (node,_) = self.get_node_item(ctx, id);

        let x = messages();
        let pc = ctx.binary.get_write_address();

        use emu::isa::AddrModeEnum::*;
        let ins_amode = ins.addr_mode;

        if ins.opcode > 0xff {
            ctx.binary
                .write_word(ins.opcode as u16)
                .map_err(|e| self.binary_error(ctx, id, e))?;
        } else {
            ctx.binary
                .write_byte(ins.opcode as u8)
                .map_err(|e| self.binary_error(ctx, id, e))?;
        }

        match ins_amode {
            Indexed => {
                if let AddrModeParseType::Indexed(imode, indirect) = amode {
                    self.compile_indexed(ctx, id, *imode, *indirect)?;
                }
            }

            Immediate8 => {
                let (arg, id) = ctx.eval.eval_first_arg(node)?;
                ctx.binary
                    .write_byte_check_size(arg)
                    .map_err(|e| self.binary_error(ctx, id, e))?;
            }

            Direct => {
                let (arg, id) = ctx.eval.eval_first_arg(node)?;
                ctx.binary
                    .write_byte_check_size(arg & 0xff)
                    .map_err(|e| self.binary_error(ctx, id, e))?;
            }

            Extended | Immediate16 => {
                let (arg, id) = ctx.eval.eval_first_arg(node)?;

                ctx.binary
                    .write_word_check_size(arg)
                    .map_err(|e| self.binary_error(ctx, id, e))?;
            }

            Relative => {
                let (arg, arg_id) = ctx.eval.eval_first_arg(node)?;
                let ( arg_n, _ ) = self.get_node_item(ctx, arg_id);
                let val = arg - (pc as i64 + ins.size as i64);
                // offset is from PC after Instruction and operand has been fetched
                use crate::binary::BinaryError::*;
                let res = ctx.binary.write_ibyte_check_size(val).map_err(|x| match x {
                    DoesNotFit { .. } => self.relative_error(ctx, id, val, 8),
                    DoesNotMatchReference { .. } => self.binary_error(ctx, id, x),
                    _ => ctx.eval.user_error(format!("{:?}", x), arg_n, false).into(),
                });

                match &res {
                    Ok(_) => (),
                    Err(_) => {
                        if self.opts.ignore_relative_offset_errors {
                            // x.warning(e.pretty().unwrap());
                            x.warning("Skipping writing relative offset");
                            ctx.binary.write_ibyte_check_size(0).unwrap();
                        } else {
                            res?;
                        }
                    }
                }
            }

            Relative16 => {
                use crate::binary::BinaryError::*;

                let (arg, arg_id) = ctx.eval.eval_first_arg(node)?;
                let ( arg_n, _ ) = self.get_node_item(ctx,arg_id);

                let val = (arg - (pc as i64 + ins.size as i64)) & 0xffff;
                // offset is from PC after Instruction and operand has been fetched
                let res = ctx.binary.write_word_check_size(val);

                res.map_err(|x| match x {
                    DoesNotFit { .. } => self.relative_error(ctx, id, val, 16),
                    DoesNotMatchReference { .. } => self.binary_error(ctx, id, x),
                    _ => ctx.eval.user_error(format!("{:?}", x), arg_n, true).into(),
                })?;
            }

            Inherent => {}

            RegisterPair => {
                if let AddrModeParseType::RegisterPair(a, b) = amode {
                    ctx.binary
                        .write_byte(reg_pair_to_flags(*a, *b))
                        .map_err(|e| self.binary_error(ctx, id, e))?;
                } else {
                    panic!("Whut!")
                }
            }

            RegisterSet => {
                let rset = &node.first_child().unwrap().value().item;

                if let Item::RegisterSet(regs) = rset {
                    let flags = registers_to_flags(regs);
                    ctx.binary
                        .write_byte(flags)
                        .map_err(|e| self.binary_error(ctx, id, e))?;
                } else {
                    panic!("Whut!")
                }
            }
        };

        let (phys_range, range) = ctx.binary.range_to_write_address(pc);
        self.add_mapping(ctx, phys_range, range, id, ItemType::OpCode);

        Ok(())
    }

    fn incbin_resolved(
        &self,
        _ctx: &mut AsmCtx,
        _id: AstNodeId,
        file: &Path,
        r: &std::ops::Range<usize>,
    ) -> GResult<()> {
        let msg = format!(
            "Including Binary {} :  offset: {:04X} len: {:04X}",
            file.to_string_lossy(),
            r.start,
            r.len()
        );

        messages().status(msg);
        panic!()

        // let bin = ctx.get_binary_chunk(file.to_path_buf(), id, r.clone())?;

        // for val in bin {
        //     binary
        //         .write_byte(val)
        //         .map_err(|e| self.binary_error(ctx,id, e))?;
        // }
        // Ok(())
    }

    fn assemble_children(&self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        let (node,_) = self.get_node_item(ctx, id);
        for c in node.children() {
            self.compile_node(ctx, c.id())?;
        }
        Ok(())
    }

    fn compile_node(&self, ctx: &mut AsmCtx, id: AstNodeId) -> GResult<()> {
        use item::Item::*;

        let (node, i) = self.get_node_item(ctx, id);

        let x = super::messages::messages();
        let pc = ctx.binary.get_write_address();
        let pos = node.value().pos.clone();

        let res: Result<(), GasmError> = try {
            match i {
                Scope(opt) => {
                    ctx.eval.symbols.set_root();
                    if opt != "root" {
                        ctx.eval.symbols.set_scope(&opt);
                    }
                }

                GrabMem => self.grab_mem(ctx, id)?,

                WriteBin(file_name) => self.write_bin(ctx, id, &file_name)?,

                IncBinRef(file_name) => {
                    self.inc_bin_ref(ctx, id, &file_name)?;
                }

                IncBinResolved { file, r } => {
                    self.incbin_resolved(ctx, id, &file, &r)?;
                }

                Skip(skip) => {
                    ctx.binary.skip(skip);
                }

                SetPc(pc) => {
                    ctx.binary.set_write_address(pc, 0);
                    x.debug(format!("Set PC to {:02X}", pc));
                }

                SetPutOffset(offset) => {
                    x.debug(format!("Set put offset to {}", offset));
                    ctx.binary.set_write_offset(offset);
                }

                OpCode(ins, amode) => {
                    self.opcode(ctx, id, &ins, &amode)?;
                }

                MacroCallProcessed { scope, macro_id } => {
                    let si = ctx.eval.get_source_info(&pos).unwrap();
                    let m = format!(
                        "{} {pos} : Expanding macro with scope {scope}",
                        si.file.to_string_lossy()
                    );

                    messages().debug(m);
                    let ( m_node, _ ) = self.get_node_item(ctx,macro_id);

                    ctx.eval.eval_macro_args(&scope, node, m_node);

                    ctx.set_scope(&scope);

                    let x = ctx.eval.symbols.get_current_scope_symbols();

                    if x.info.is_empty() {
                        println!("EMPTY");
                    }

                    for s in x.info.values() {
                        println!("   {} {}", s.name, s.value.unwrap());
                    }

                    self.assemble_children(ctx, macro_id)?;
                    ctx.pop_scope();
                }

                Block | TokenizedFile(..) => {
                    self.assemble_children(ctx, id)?;
                }

                Fdb(..) => {
                    for n in node.children() {
                        let x = ctx.eval.eval_node(n)?;
                        ctx.binary
                            .write_word_check_size(x)
                            .map_err(|e| self.binary_error(ctx, n.id(), e))?;
                    }

                    let (phys_range, range) = ctx.binary.range_to_write_address(pc);
                    self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
                }

                Fcb(..) => {
                    for n in node.children() {
                        let x = ctx.eval.eval_node(n)?;
                        ctx.binary
                            .write_byte_check_size(x)
                            .map_err(|e| self.binary_error(ctx, n.id(), e))?;
                    }
                    let (phys_range, range) = ctx.binary.range_to_write_address(pc);
                    self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
                }

                Fcc(text) => {
                    for c in text.as_bytes() {
                        ctx.binary
                            .write_byte(*c)
                            .map_err(|e| self.binary_error(ctx, id, e))?;
                    }
                    let (phys_range, range) = ctx.binary.range_to_write_address(pc);
                    self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
                }

                Zmb => {
                    let (bytes, _) = ctx.eval.eval_first_arg(node)?;
                    for _ in 0..bytes {
                        ctx.binary
                            .write_byte(0)
                            .map_err(|e| self.binary_error(ctx, id, e))?;
                    }
                    let (phys_range, range) = ctx.binary.range_to_write_address(pc);
                    self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
                }

                Zmd => {
                    let (words, _) = ctx.eval.eval_first_arg(node)?;
                    for _ in 0..words {
                        ctx.binary
                            .write_word(0)
                            .map_err(|e| self.binary_error(ctx, id, e))?;
                    }

                    let (phys_range, range) = ctx.binary.range_to_write_address(pc);
                    self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
                }

                Fill => {
                    let (byte, size) = ctx.eval.eval_two_args(node)?;

                    for _ in 0..size {
                        ctx.binary
                            .write_ubyte_check_size(byte)
                            .map_err(|e| self.binary_error(ctx, id, e))?;
                    }

                    let (phys_range, range) = ctx.binary.range_to_write_address(pc);
                    self.add_mapping(ctx, phys_range, range, id, ItemType::Command);
                }

                IncBin(..) | Org | AssignmentFromPc(..) | Assignment(..) | Comment(..) | Rmb
                | StructDef(..) | MacroDef(..) | MacroCall(..) | SetDp => (),

                _ => {
                    panic!("Unable to assemble {:?}", i);
                }
            }

            let (_, phys_range) = ctx.binary.range_to_write_address(pc);

            if phys_range.len() != 0 {
                if let Ok(si) = ctx.eval.get_source_info(&node.value().pos) {
                    let m_pc = format!(
                        "{:05X} {:04X} {:02X?} ",
                        phys_range.start,
                        pc,
                        ctx.binary.get_bytes(phys_range.start, phys_range.len())
                    );
                    let m = format!("{:50}{}", m_pc, si.line_str);
                    if m.len() < 100 {
                        messages().debug(m);
                    }
                }
            }
        };

        let ret = match res {
            Ok(_) => Ok(()),
            Err(_e) => {
                panic!();
                // self.ctx.ctx.errors.add_error(e, false);
            }
        };

        ret
    }
}
