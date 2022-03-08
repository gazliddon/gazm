use clap::Indices;
use colored::*;
use emu::cpu::RegEnum;
use emu::isa::AddrModeEnum;
use nom::combinator::map_opt;
use nom::combinator::recognize;
use nom::Offset;
use romloader::sources::{
    FileIo, SourceFileLoader, SourceInfo, SymbolError, SymbolQuery, SymbolWriter,
};
use serde_json::to_string;

use crate::assemble;
use crate::ast::AstNodeRef;
use crate::ast::AstTree;
use crate::ast::{Ast, AstNodeId, AstNodeMut};
use crate::astformat::as_string;
use crate::binary::BinRef;
use crate::binary::{AccessType, Binary};
use crate::cli;
use crate::ctx::Context;
use crate::error;
use crate::error::AstError;
use crate::error::UserError;
use crate::eval;
use crate::eval::eval;
use crate::item;
use crate::item::AddrModeParseType;
use crate::item::IndexParseType;
use crate::messages::Verbosity;
use crate::messages::info;
use crate::messages::messages;
use crate::util;
use crate::util::ByteSize;
use item::{Item, Node};
use romloader::sources::{ItemType, Position, SourceDatabase, SourceMapping, Sources, SymbolTable};
use std::collections::HashSet;
use std::net::UdpSocket;
use std::ops::RangeBounds;
use std::path::Path;
use std::path::PathBuf;
use std::vec;

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Assembled {
    #[serde(skip)]
    pub mem: Vec<u8>,
    pub database: SourceDatabase,
}

fn reg_to_reg_num(a: RegEnum) -> u8 {
    use emu::cpu::RegEnum::*;

    match a {
        D => 0b0000,
        X => 0b0001,
        Y => 0b0010,
        U => 0b0011,
        S => 0b0100,
        PC => 0b0101,
        A => 0b1000,
        B => 0b1001,
        CC => 0b1010,
        DP => 0b1011,
    }
}

fn reg_pair_to_flags(source: RegEnum, dest: RegEnum) -> u8 {
    let a = reg_to_reg_num(source);
    let b = reg_to_reg_num(dest);
    (a << 4) | b
}

fn registers_to_flags(regs: &HashSet<RegEnum>) -> u8 {
    use emu::cpu::RegEnum::*;
    let mut registers = 0;

    if regs.contains(&CC) {
        registers |= 0x01;
    }

    if regs.contains(&A) {
        registers |= 0x02;
    }
    if regs.contains(&B) {
        registers |= 0x04;
    }

    if regs.contains(&DP) {
        registers |= 0x08;
    }

    if regs.contains(&X) {
        registers |= 0x10;
    }

    if regs.contains(&Y) {
        registers |= 0x20;
    }

    if regs.contains(&D) {
        registers |= 0x02;
        registers |= 0x04;
    }

    if regs.contains(&U) || regs.contains(&S) {
        registers |= 0x40;
    }

    if regs.contains(&PC) {
        registers |= 0x80;
    }
    registers
}

pub struct Assembler<'a> {
    binary: Binary,
    tree: crate::ast::AstTree,
    source_map: SourceMapping,
    direct_page: Option<u8>,
    ctx: &'a mut crate::ctx::Context,
}

fn eval_node(symbols: &SymbolTable, node: AstNodeRef, sources: &Sources) -> Result<i64, UserError> {
    eval(symbols, node).map_err(|err| {
        let info = sources.get_source_info(&node.value().pos).unwrap();
        let ast_err: AstError = err.into();
        UserError::from_ast_error(ast_err, &info)
    })
}

use thiserror::Error;

#[derive(Error, Debug)]
enum AssemblerErrors {
    #[error("Could not load MapFile {0}")]
    MapFile(String),
    #[error("Could not load binary reference file {0}")]
    BinRefFile(String),
    #[error("Could not add symbols")]
    AddingSymbols,
}

impl<'a> Assembler<'a> {
    pub fn new(ast: crate::ast::Ast<'a>) -> Result<Self, Box<dyn std::error::Error>> {
        let ctx = ast.ctx;

        let mut binary = Binary::new(ctx.memory_image_size, AccessType::ReadWrite);

        if let Some(file) = &ctx.as6809_lst {
            messages().status(format!("Loading map file {}", file));
            let m = crate::as6809::MapFile::new(&file)
                .map_err(|_| AssemblerErrors::MapFile(file.to_string()))?;
            binary.addr_reference(m);
        }

        if let Some(file) = &ctx.as6809_sym {
            crate::as6809::add_reference_syms(file, &mut ctx.symbols)
                .map_err(|_| AssemblerErrors::AddingSymbols)?;
        }

        let ret = Self {
            binary,
            tree: ast.tree,
            source_map: SourceMapping::new(),
            direct_page: None,
            ctx,
        };

        Ok(ret)
    }

    pub fn set_dp(&mut self, dp: i64) {
        if dp < 0 {
            self.direct_page = None
        } else {
            self.direct_page = Some(dp as u64 as u8)
        }
    }

    fn get_source_info(&self, pos: &Position) -> Result<SourceInfo, String> {
        self.ctx.sources().get_source_info(pos)
    }

    fn binary_error(&self, n: AstNodeId, e: crate::binary::BinaryError) -> UserError {
        let n = self.tree.get(n).unwrap();
        let info = &self.get_source_info(&n.value().pos).unwrap();
        let msg = e.to_string();
        UserError::from_text(msg, info, true)
    }

    fn relative_error(&self, n: AstNodeId, val: i64, bits: usize) -> UserError {
        let p = 1 << (bits - 1);

        let message = if val < 0 {
            format!("Branch out of range by {} bytes ({val})", (p + val).abs())
        } else {
            format!("Branch out of range by {} bytes ({val})", val - (p - 1))
        };

        let n = self.tree.get(n).unwrap();
        let info = &self.get_source_info(&n.value().pos).unwrap();
        let msg = message;
        UserError::from_text(msg, info, true)
    }

    pub fn size(&mut self) -> Result<(), UserError> {
        let root = self.tree.root().id();
        info("Sizing items", |_| {
            self.size_node(0, root)?;
            Ok(())
        })
    }

    fn set_item(&mut self, node_id: AstNodeId, value: item::Item) {
        let mut node_mut = self.tree.get_mut(node_id).unwrap();
        node_mut.value().item = value;
    }

    fn user_error<S: Into<String>>(&self, err: S, id: AstNodeId, is_failure: bool) -> UserError {
        let node = self.tree.get(id).unwrap();
        let info = self.get_source_info(&node.value().pos).unwrap();
        UserError::from_text(err, &info, is_failure)
    }
    fn user_error_id<S: Into<String>>(
        &self,
        err: S,
        node: AstNodeId,
        is_failure: bool,
    ) -> UserError {
        let node = self.tree.get(node).unwrap();
        let info = self.get_source_info(&node.value().pos).unwrap();
        UserError::from_text(err, &info, is_failure)
    }

    fn eval_node(&self, node: AstNodeRef) -> Result<i64, UserError> {
        eval(&self.ctx.symbols, node).map_err(|err| {
            let info = self.get_source_info(&node.value().pos).unwrap();
            UserError::from_ast_error(err.into(), &info)
        })
    }

    fn eval_first_arg(&self, id: AstNodeId) -> Result<(i64, AstNodeId), UserError> {
        let node = self.tree.get(id).unwrap();
        let c = node
            .first_child()
            .ok_or_else(|| self.user_error("Missing argument", id, true))?;
        let v = self.eval_node(c)?;
        Ok((v, c.id()))
    }

    fn eval_two_args(&self, id: AstNodeId) -> Result<(i64, i64), UserError> {
        let args = self.eval_n_args(id, 2)?;
        Ok((args[0], args[1]))
    }

    fn eval_n_args(&self, id: AstNodeId, n: usize) -> Result<Vec<i64>, UserError> {
        let node = self.tree.get(id).unwrap();
        let mut ret = vec![];

        for (i, node) in node.children().enumerate() {
            if i == n {
                break;
            }
            let v = self.eval_node(node)?;
            ret.push(v)
        }

        Ok(ret)
    }

    pub fn assemble(&mut self) -> Result<Assembled, UserError> {
        self.assemble_node(self.tree.root().id())?;

        let x = self.binary.get_unchecked_writes();

        let database =
            SourceDatabase::new(&self.source_map, &self.ctx.sources(), &self.ctx.symbols);

        for uc in x {
            let text = if let Some(si) = database.get_source_info_from_physical_address(uc.physical)
            {
                format!(
                    "{} {} {}",
                    si.file.to_string_lossy(),
                    si.line_number,
                    si.text
                )
            } else {
                "no info!".to_string()
            };

            println!("{:05X?} {}", uc, text);
        }

        let ret = Assembled {
            mem: self.binary.data.clone(),
            database,
        };

        Ok(ret)
    }

    fn assemble_indexed(
        &mut self,
        id: AstNodeId,
        imode: IndexParseType,
        indirect: bool,
    ) -> Result<(), UserError> {
        let idx_byte = imode.get_index_byte(indirect);

        let n = self.tree.get(id).unwrap();

        let si = self.get_source_info(&n.value().pos).unwrap();

        messages().debug(format!("{} {:?}", si.line_str, imode));

        self.binary
            .write_byte(idx_byte)
            .map_err(|e| self.binary_error(id, e))?;

        use item::IndexParseType::*;

        match imode {
            PCOffset | ConstantOffset(..) => {
                panic!("Should not happen")
            }

            ExtendedIndirect => {
                let (val, _) = self.eval_first_arg(id)?;
                self.binary
                    .write_uword_check_size(val)
                    .map_err(|e| self.binary_error(id, e))?;
            }

            ConstantWordOffset(_, val) | PcOffsetWord(val) => {
                self.binary
                    .write_iword_check_size(val as i64)
                    .map_err(|e| self.binary_error(id, e))?;
            }

            ConstantByteOffset(_, val) | PcOffsetByte(val) => {
                self.binary
                    .write_ibyte_check_size(val as i64)
                    .map_err(|e| self.binary_error(id, e))?;
            }
            _ => (),
        }

        Ok(())
    }

    /// Adds a mapping of this source file fragment to a physicl and logical range of memory
    /// ( physical range, logical_range )
    fn add_mapping(
        &mut self,
        phys_range: std::ops::Range<usize>,
        range: std::ops::Range<usize>,
        pos: &Position,
        i: ItemType,
    ) {
        self.source_map
            .add_mapping(&self.ctx.sources(), phys_range, range, pos, i);
    }

    /// Returns ranges from current_pc -> pc
    /// ( physical range, logical_range )
    fn get_range(&self, pc: i64) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
        let start = pc as usize;
        let end = self.get_pc();

        let pstart = self.binary.logical_to_physical(start);
        let pend = self.binary.logical_to_physical(end);

        (start..end, pstart..pend)
    }

    // Get the PC we're using to assemble to
    fn get_pc(&self) -> usize {
        self.binary.get_write_address()
    }

    fn assemble_node(&mut self, id: AstNodeId) -> Result<(), UserError> {
        use item::Item::*;
        let x = super::messages::messages();

        let node = self.tree.get(id).unwrap();
        let id = node.id();
        let i = &node.value().item.clone();
        let pos = &node.value().pos.clone();

        let pc = self.get_pc() as i64;

        match i {
            Scope(opt) => {
                self.set_scope(opt);
            }

            GrabMem => {
                let args = self.eval_n_args(id, 2)?;
                let source = args[0];
                let size = args[1];
                let bytes = self
                    .binary
                    .get_bytes(source as usize, size as usize)
                    .to_vec();
                self.binary
                    .write_bytes(&bytes)
                    .map_err(|e| self.binary_error(id, e))?;
            }

            WriteBin(file_name) => {
                let (addr, size) = self.eval_two_args(id)?;
                let mem = self.binary.get_bytes(addr as usize, size as usize);
                let p = self.ctx.write(file_name, mem);
                messages().info(format!(
                    "Write mem: {} {addr:05X} {size:05X}",
                    p.to_string_lossy()
                ));
            }

            IncBinRef(file_name) => {
                let data = self.get_binary(file_name, id)?;
                let dest = self.binary.get_write_location().physical;

                let bin_ref = BinRef {
                    file: file_name.to_path_buf(),
                    start: 0,
                    size: data.len(),
                    dest,
                };

                self.binary.bin_reference(&bin_ref, &data);
                let file_name = file_name.to_string_lossy();

                messages().info(format!(
                    "Adding binary reference {file_name} for {:05X} - {:05X}",
                    dest,
                    dest + data.len()
                ));
            }

            IncBinResolved { file, r } => {
                let msg = format!(
                    "Including Binary {} :  offset: {:04X} len: {:04X}",
                    file.to_string_lossy(),
                    r.start,
                    r.len()
                );

                messages().status(msg);

                let bin = self.get_binary_chunk(file.to_path_buf(), id, r.clone())?;

                for val in bin {
                    self.binary
                        .write_byte(val)
                        .map_err(|e| self.binary_error(id, e))?;
                }
            }

            Skip(skip) => {
                self.binary.skip(*skip);
            }

            SetPc(pc) => {
                self.binary.set_write_address(*pc as usize, 0);
                x.debug(format!("Set PC to {:02X}", *pc));
            }

            SetPutOffset(offset) => {
                x.debug(format!("Set put offset to {}", offset));
                self.binary.set_write_offset(*offset);
            }

            OpCode(ins, amode) => {
                use emu::isa::AddrModeEnum::*;
                let ins_amode = ins.addr_mode;

                if ins.opcode > 0xff {
                    self.binary
                        .write_word(ins.opcode)
                        .map_err(|e| self.binary_error(id, e))?;
                } else {
                    self.binary
                        .write_byte(ins.opcode as u8)
                        .map_err(|e| self.binary_error(id, e))?;
                }

                match ins_amode {
                    Indexed => {
                        if let AddrModeParseType::Indexed(imode, indirect) = amode {
                            self.assemble_indexed(id, *imode, *indirect)?;
                        }
                    }

                    Immediate8 | Direct => {
                        let (arg, id) = self.eval_first_arg(id)?;
                        self.binary
                            .write_byte_check_size(arg)
                            .map_err(|e| self.binary_error(id, e))?;
                    }

                    Extended | Immediate16 => {
                        let (arg, id) = self.eval_first_arg(id)?;

                        self.binary
                            .write_word_check_size(arg)
                            .map_err(|e| self.binary_error(id, e))?;
                    }

                    Relative => {
                        let (arg, id) = self.eval_first_arg(id)?;
                        let val = arg - (pc + ins.size as i64);
                        // offset is from PC after Instruction and operand has been fetched
                        use crate::binary::BinaryError::*;
                        let res = self
                            .binary
                            .write_ibyte_check_size(val)
                            .map_err(|x| match x {
                                DoesNotFit { .. } => self.relative_error(id, val, 8),
                                DoesNotMatchReference { .. } => self.binary_error(id, x),
                                _ => self.user_error(format!("{:?}", x), id, true),
                            });

                        match &res {
                            Ok(_) => (),
                            Err(e) => {
                                if self.ctx.ignore_relative_offset_errors {
                                    x.warning(e.pretty().unwrap());
                                    x.warning("Skipping writing relative offset");
                                    self.binary.write_ibyte_check_size(0).unwrap();
                                } else {
                                    res?;
                                }
                            }
                        }
                    }

                    Relative16 => {
                        use crate::binary::BinaryError::*;
                        let (arg, id) = self.eval_first_arg(id)?;
                        let val = arg - (pc + ins.size as i64);
                        // offset is from PC after Instruction and operand has been fetched
                        let res = self.binary.write_iword_check_size(val);

                        res.map_err(|x| match x {
                            DoesNotFit { .. } => self.relative_error(id, val, 16),
                            DoesNotMatchReference { .. } => self.binary_error(id, x),
                            _ => self.user_error(format!("{:?}", x), id, true),
                        })?;
                    }

                    Inherent => {}

                    RegisterPair => {
                        if let AddrModeParseType::RegisterPair(a, b) = amode {
                            self.binary
                                .write_byte(reg_pair_to_flags(*a, *b))
                                .map_err(|e| self.binary_error(id, e))?;
                        } else {
                            panic!("Whut!")
                        }
                    }

                    RegisterSet => {
                        let rset = &node.first_child().unwrap().value().item;

                        if let Item::RegisterSet(regs) = rset {
                            self.binary
                                .write_byte(registers_to_flags(regs))
                                .map_err(|e| self.binary_error(id, e))?;
                        } else {
                            panic!("Whut!")
                        }
                    }
                };

                let (phys_range, range) = self.get_range(pc);
                self.add_mapping(phys_range, range, pos, ItemType::OpCode);
            }

            ExpandedMacro(mcall) => {
                // We need to tell the source mapper we're expanding a macro so the file / line for
                // everything expanded by the macro will point to the line that instantiated the
                //
                let si = self.get_source_info(&mcall.name).unwrap();
                let frag = si.fragment.to_string();
                self.source_map.start_macro(&mcall.name);

                let children: Vec<_> = node.children().map(|n| n.id()).collect();
                for c in children {
                    let mut res = self.assemble_node(c);

                    if let Err(ref mut err) = res {
                        err.message = format!("expanding macro : {}\n{}", frag, err.message);
                    }

                    res?;
                }

                self.source_map.stop_macro();
            }

            Block | TokenizedFile(..) => {
                let children: Vec<_> = node.children().map(|n| n.id()).collect();
                for c in children {
                    self.assemble_node(c)?;
                }
            }

            Fdb(..) => {
                for n in node.children() {
                    let x = self.eval_node(n)?;
                    self.binary
                        .write_word_check_size(x)
                        .map_err(|e| self.binary_error(n.id(), e))?;
                }

                let (phys_range, range) = self.get_range(pc);
                self.add_mapping(phys_range, range, pos, ItemType::Command);
            }

            Fcb(..) => {
                for n in node.children() {
                    let x = self.eval_node(n)?;
                    self.binary
                        .write_byte_check_size(x)
                        .map_err(|e| self.binary_error(n.id(), e))?;
                }
                let (phys_range, range) = self.get_range(pc);
                self.add_mapping(phys_range, range, pos, ItemType::Command);
            }

            Fcc(text) => {
                for c in text.as_bytes() {
                    self.binary
                        .write_byte(*c)
                        .map_err(|e| self.binary_error(id, e))?;
                }
                let (phys_range, range) = self.get_range(pc);
                self.add_mapping(phys_range, range, pos, ItemType::Command);
            }

            Zmb => {
                let (bytes, _) = self.eval_first_arg(id)?;
                for _ in 0..bytes {
                    self.binary
                        .write_byte(0)
                        .map_err(|e| self.binary_error(id, e))?;
                }
                let (phys_range, range) = self.get_range(pc);
                self.add_mapping(phys_range, range, pos, ItemType::Command);
            }

            Zmd => {
                let (words, _) = self.eval_first_arg(id)?;
                for _ in 0..words {
                    self.binary
                        .write_word(0)
                        .map_err(|e| self.binary_error(id, e))?;
                }

                let (phys_range, range) = self.get_range(pc);
                self.add_mapping(phys_range, range, pos, ItemType::Command);
            }

            Fill => {
                let (byte, size) = self.eval_two_args(id)?;

                for _ in 0..size {
                    self.binary
                        .write_ubyte_check_size(byte)
                        .map_err(|e| self.binary_error(id, e))?;
                }

                let (phys_range, range) = self.get_range(pc);
                self.add_mapping(phys_range, range, pos, ItemType::Command);
            }

            IncBin(..) | Org | AssignmentFromPc(..) | Assignment(..) | Comment(..)
            | MacroDef(..) | Rmb | StructDef(..) => (),

            SetDp => {
                let (dp, _) = self.eval_first_arg(id)?;
                self.set_dp(dp);
            }

            _ => {
                panic!("Unable to assemble {:?}", i);
            }
        }

        Ok(())
    }

    fn size_indexed(&mut self, mut pc: u64, id: AstNodeId) -> Result<u64, UserError> {
        use crate::util::{ByteSize, ByteSizes};
        use item::Item::*;
        let node = self.tree.get(id).unwrap();
        let i = &node.value().item;

        if let OpCode(ins, amode) = i {
            if let AddrModeParseType::Indexed(pmode, indirect) = amode {
                pc += ins.size as u64;
                let indirect = *indirect;
                use item::IndexParseType::*;

                match pmode {
                    Zero(..) | AddA(..) | AddB(..) | AddD(..) | Plus(..) | PlusPlus(..)
                    | Sub(..) | SubSub(..) => (),

                    ConstantByteOffset(..)
                    | PcOffsetByte(..)
                    | PcOffsetWord(..)
                    | ConstantWordOffset(..)
                    | Constant5BitOffset(..) => {
                        panic!()
                    }

                    ConstantOffset(r) => {
                        let (v, _) = self.eval_first_arg(id)?;

                        let mut bs = v.byte_size();

                        if let ByteSizes::Bits5(val) = bs {
                            if indirect {
                                // Indirect constant offset does not support
                                // 5 bit offsets so promote to 8 bit
                                bs = ByteSizes::Byte(val);
                            }
                        }

                        let new_amode = match bs {
                            ByteSizes::Zero => Zero(*r),
                            ByteSizes::Bits5(v) => Constant5BitOffset(*r, v),
                            ByteSizes::Word(v) => {
                                pc += 2;
                                ConstantWordOffset(*r, v)
                            }
                            ByteSizes::Byte(v) => {
                                pc += 1;
                                ConstantByteOffset(*r, v)
                            }
                        };

                        let ins = ins.clone();
                        self.set_item(
                            id,
                            OpCode(ins, AddrModeParseType::Indexed(new_amode, indirect)),
                        );
                    }

                    PCOffset => {
                        let (v, id) = self.eval_first_arg(id)?;

                        let new_amode = match v.byte_size() {
                            ByteSizes::Zero => {
                                pc += 1;
                                PcOffsetByte(0)
                            }

                            ByteSizes::Bits5(v) | ByteSizes::Byte(v) => {
                                pc += 1;
                                PcOffsetByte(v)
                            }
                            ByteSizes::Word(v) => {
                                pc += 2;
                                PcOffsetWord(v)
                            }
                        };
                        let ins = ins.clone();

                        self.set_item(
                            id,
                            OpCode(ins, AddrModeParseType::Indexed(new_amode, indirect)),
                        );
                    }

                    ExtendedIndirect => pc += 2,
                };
                return Ok(pc);
            }
        }

        panic!();
    }

    fn eval_with_pc(&mut self, n: AstNodeId, pc: u64) -> Result<i64, UserError> {
        let n = self.tree.get(n).unwrap();
        self.ctx
            .symbols
            .add_symbol_with_value("*", pc as i64)
            .unwrap();
        let ret = self.eval_node(n)?;
        self.ctx.symbols.remove_symbol_name("*");
        Ok(ret)
    }

    fn set_scope(&mut self, scope: &str) {
        self.ctx.symbols.set_root();
        if scope != "root".to_string() {
            self.ctx.symbols.set_scope(scope);
        }
    }

    fn get_binary_extents(
        &self,
        file_name: PathBuf,
        id: AstNodeId,
    ) -> Result<std::ops::Range<usize>, UserError> {
        use Item::*;

        let data_len = self
            .ctx
            .get_size(file_name)
            .map_err(|e| self.user_error(e.to_string(), id, true))?;

        let node = self.tree.get(id).unwrap();

        let mut r = 0..data_len;

        let mut c = node.children();

        let offset_size = c
            .next()
            .and_then(|offset| c.next().map(|size| (offset, size)));

        if let Some((offset, size)) = offset_size {
            let offset = self.eval_node(offset)?;
            let size = self.eval_node(size)?;
            let offset = offset as usize;
            let size = size as usize;
            let last = (offset + size) - 1;

            if !(r.contains(&offset) && r.contains(&last)) {
                let msg =
                    format!("Trying to grab {offset:04X} {size:04X} from file size {data_len:X}");
                return Err(self.user_error(msg, id, true));
            };

            r.start = offset;
            r.end = offset + size;
        }

        Ok(r)
    }

    fn get_binary_chunk(
        &mut self,
        file_name: PathBuf,
        id: AstNodeId,
        range: std::ops::Range<usize>,
    ) -> Result<Vec<u8>, UserError> {
        let (_, bin) = self
            .ctx
            .read_binary_chunk(file_name, range.clone())
            .map_err(|e| self.user_error(e.to_string(), id, true))?;

        Ok(bin)
    }

    fn get_binary(&mut self, file_name: &Path, id: AstNodeId) -> Result<Vec<u8>, UserError> {
        let range = self.get_binary_extents(file_name.to_path_buf(), id)?;
        self.get_binary_chunk(file_name.to_path_buf(), id, range)
    }

    fn size_node(&mut self, mut pc: u64, id: AstNodeId) -> Result<u64, UserError> {
        use crate::util::{ByteSize, ByteSizes};
        use item::Item::*;

        use crate::astformat;

        let x_node = self.tree.get(id).unwrap();
        let _pos = x_node.value().pos.clone();
        let id = x_node.id().clone();
        let i = x_node.value().item.clone();

        match &i {
            Scope(opt) => {
                self.set_scope(opt);
            }

            GrabMem => {
                let args = self.eval_n_args(id, 2)?;
                let size = args[1];
                pc = pc + size as u64;
            }

            Org => {
                let (value, _) = self.eval_first_arg(id)?;
                self.set_item(id, Item::SetPc(value as u16));
                pc = value as u64;
            }

            Put => {
                // let x = messages();
                let (value, _) = self.eval_first_arg(id)?;
                let offset = (value - pc as i64) as isize;
                self.set_item(id, Item::SetPutOffset(offset));
            }

            Rmb => {
                let (bytes, _) = self.eval_first_arg(id)?;

                if bytes < 0 {
                    return Err(self.user_error("Argument for RMB must be positive", id, true));
                };

                self.set_item(id, Item::Skip(bytes as usize));

                pc = pc + bytes as u64;
            }

            OpCode(ins, amode) => {
                use emu::isa::AddrModeEnum::*;

                match amode {
                    AddrModeParseType::Extended(false) => {
                        // If there is a direct page set AND
                        // I can evaluate the arg AND
                        // the instruction supports DIRECT addressing (phew)
                        // I can changing this to a direct page mode instruction
                        // !!!! and it wasn't forced (need someway to propogate this from parse)

                        let mut size = ins.size;

                        use crate::opcodes::get_opcode_info;

                        let dp_info = get_opcode_info(&ins)
                            .and_then(|i_type| i_type.get_instruction(&AddrModeEnum::Direct))
                            .and_then(|ins| self.direct_page.map(|dp| (ins, dp)));

                        if let Some((new_ins, dp)) = dp_info {
                            if let Ok((value, _)) = self.eval_first_arg(id) {
                                let top_byte = ((value >> 8) & 0xff) as u8;

                                if top_byte == dp {

                                    // Here we go!
                                    let new_ins = new_ins.clone();
                                    size = new_ins.size;
                                    // get the node
                                    let mut node_mut = self.tree.get_mut(id).unwrap();
                                    // Change the opcode to the direct version
                                    node_mut.value().item =
                                        OpCode(new_ins, AddrModeParseType::Direct);
                                    // Change the value to just include the lower 8 bits
                                    node_mut.first_child().unwrap().value().item =
                                        Item::Number(value & 0xff);
                                }
                            } 
                        }
                        pc += size as u64;
                    }

                    AddrModeParseType::Indexed(..) => {
                        pc = self.size_indexed(pc, id)?;
                    }

                    _ => {
                        pc += ins.size as u64;
                    }
                };
            }

            AssignmentFromPc(name) => {
                let node = self.tree.get(id).unwrap();

                let pcv = if let Some(n) = node.first_child() {
                    self.ctx
                        .symbols
                        .add_symbol_with_value("*", pc as i64)
                        .unwrap();
                    let ret = self.eval_node(n)?;
                    self.ctx.symbols.remove_symbol_name("*");
                    ret
                } else {
                    pc as i64
                };

                self.ctx
                    .symbols
                    .add_symbol_with_value(name, pcv)
                    .map_err(|e| {
                        let err = if let SymbolError::Mismatch { expected } = e {
                            format!(
                                "Mismatch symbol {name} : expected {:04X} got : {:04X}",
                                expected, pcv
                            )
                        } else {
                            format!("{:?}", e)
                        };
                        self.user_error_id(err, id, false)
                    })?;
            }

            Block | ExpandedMacro(..) | TokenizedFile(..) => {
                let node = self.tree.get(id).unwrap();
                let children: Vec<_> = node.children().map(|n| n.id()).collect();
                for c in children {
                    pc = self.size_node(pc, c)?;
                }
            }

            Fdb(num_of_words) => {
                pc += (*num_of_words * 2) as u64;
            }

            Fcb(num_of_bytes) => {
                pc += *num_of_bytes as u64;
            }

            Fcc(text) => {
                pc += text.as_bytes().len() as u64;
            }

            Zmb => {
                let (v, _) = self.eval_first_arg(id)?;
                assert!(v >= 0);
                pc += v as u64;
            }

            Zmd => {
                let (v, _) = self.eval_first_arg(id)?;
                assert!(v >= 0);
                pc += (v * 2) as u64;
            }

            Fill => {
                let (_, c) = self.eval_two_args(id)?;
                assert!(c >= 0);
                pc += c as u64;
            }

            SetDp => {
                let (dp, _) = self.eval_first_arg(id)?;
                self.set_dp(dp);
            }

            IncBin(file_name) => {
                let r = self.get_binary_extents(file_name.to_path_buf(), id)?;

                pc = pc + r.len() as u64;
                let new_item = IncBinResolved {
                    file: file_name.to_path_buf(),
                    r,
                };
                let mut node_mut = self.tree.get_mut(id).unwrap();
                node_mut.value().item = new_item;
            }

            WriteBin(..) | IncBinRef(..) | Assignment(..) | Comment(..) | MacroDef(..)
            | StructDef(..) => (),
            _ => {
                let i = &self.tree.get(id).unwrap().value().item;
                let msg = format!("Unable to size {:?}", i);
                return Err(self.user_error(msg, id, true));
            }
        };

        Ok(pc)
    }
}
