use clap::Indices;
use colored::*;
use emu::cpu::RegEnum;
use emu::isa::AddrModeEnum;
use nom::combinator::map_opt;
use nom::combinator::recognize;
use romloader::sources::SymbolError;

use crate::assemble;
use crate::ast::AstNodeRef;
use crate::ast::AstTree;
use crate::ast::{Ast, AstNodeId, AstNodeMut};
use crate::astformat::as_string;
use crate::binary::Binary;
use crate::cli;
use crate::cli::Context;
use crate::error::UserError;
use crate::eval;
use crate::eval::eval;
use crate::item;
use crate::item::AddrModeParseType;
use crate::item::IndexParseType;
use crate::messages::info;
use crate::messages::messages;
use crate::util;
use crate::util::ByteSize;
use item::{Item, Node};
use romloader::sources::{ItemType, SourceDatabase, SourceMapping, Sources, SymbolTable};
use romloader::ResultExt;
use std::collections::HashSet;
use std::net::UdpSocket;
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

pub struct Assembler {
    symbols: SymbolTable,
    sources: Sources,
    binary: Binary,
    tree: crate::ast::AstTree,
    source_map: SourceMapping,
    direct_page: Option<u8>,
    ctx: crate::cli::Context,
}

fn eval_node(symbols: &SymbolTable, node: AstNodeRef, sources: &Sources) -> Result<i64, UserError> {
    eval(symbols, node).map_err(|err| {
        let info = sources.get_source_info(&node.value().pos).unwrap();
        UserError::from_ast_error(err, &info)
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

impl Assembler {
    pub fn new(ast: crate::ast::Ast, ctx: &Context) -> Result<Self, Box<dyn std::error::Error>> {
        let mut binary = Binary::new();

        if let Some(file) = &ctx.as6809_lst {
            messages().status(format!("Loading map file {}", file));
            let m = crate::as6809::MapFile::new(&file).map_err(|_| 
                AssemblerErrors::MapFile(file.to_string())
                )?;
            binary.addr_reference(m);
        }

        for bin_ref in &ctx.bin_refs {
            messages().status(format!(
                "Adding bin reference file {}",
                bin_ref.file.clone().to_string_lossy()
            ));

            use std::fs::File;
            use std::io::Read;
            let mut buffer = Vec::new();
            let filename = bin_ref.file.to_string_lossy();
            let mut file = File::open(&bin_ref.file).map_err(|_|
                AssemblerErrors::BinRefFile(filename.to_string()))?;

            file.read_to_end(&mut buffer).map_err(|_|
                AssemblerErrors::BinRefFile(filename.to_string()))?;

            binary.bin_reference(
                bin_ref.dest,
                &buffer[bin_ref.start..(bin_ref.start + bin_ref.size)],
            );
        }

        let mut symbols = ast.symbols;

        if let Some(file) = &ctx.as6809_sym {
            crate::as6809::add_reference_syms(file, &mut symbols).map_err(|_| AssemblerErrors::AddingSymbols)?;
        }

        let ret = Self {
            symbols,
            sources: ast.sources,
            binary,
            tree: ast.tree,
            source_map: SourceMapping::new(),
            direct_page: None,
            ctx: ctx.clone(),
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

    pub fn assemble_indexed_opcode(
        &mut self,
        _ins: &emu::isa::Instruction,
        _addr_mode: &AddrModeParseType,
        _node: AstNodeRef,
    ) -> Result<(), UserError> {
        todo!("assemble indexed opcode")
    }

    fn binary_error(&self, n: AstNodeId, e: crate::binary::BinaryError) -> UserError {
        let n = self.tree.get(n).unwrap();
        let info = &self.sources.get_source_info(&n.value().pos).unwrap();
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
        let info = &self.sources.get_source_info(&n.value().pos).unwrap();
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

    fn user_error<S: Into<String>>(&self, err: S, node: AstNodeRef, is_failure: bool) -> UserError {
        let info = self.sources.get_source_info(&node.value().pos).unwrap();
        UserError::from_text(err, &info, is_failure)
    }

    fn eval_node(&self, node: AstNodeRef) -> Result<i64, UserError> {
        eval(&self.symbols, node).map_err(|err| {
            let info = self.sources.get_source_info(&node.value().pos).unwrap();
            UserError::from_ast_error(err, &info)
        })
    }

    fn eval_first_arg(&self, node: AstNodeRef) -> Result<(i64, AstNodeId), UserError> {
        let c = node
            .first_child()
            .ok_or_else(|| self.user_error("Missing argument", node, true))?;
        let v = self.eval_node(c)?;
        Ok((v, c.id()))
    }

    fn eval_two_args(&self, node: AstNodeRef) -> Result<(i64, i64), UserError> {
        let args = self.eval_n_args(node, 2)?;
        Ok((args[0], args[1]))
    }

    fn eval_n_args(&self, node: AstNodeRef, n: usize) -> Result<Vec<i64>, UserError> {
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

        let database = SourceDatabase::new(&self.source_map, &self.sources, &self.symbols);

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

        let si = self.sources.get_source_info(&n.value().pos).unwrap();

        messages().debug(format!("{} {:?}", si.line_str, imode));

        self.binary
            .write_byte(idx_byte)
            .map_err(|e| self.binary_error(id, e))?;

        use item::IndexParseType::*;
        let node = self.tree.get(id).unwrap();

        match imode {
            PCOffset | ConstantOffset(..) => {
                panic!("Should not happen")
            }

            ExtendedIndirect => {
                let (val, _) = self.eval_first_arg(node)?;
                self.binary
                    .write_uword_check_size(val)
                    .map_err(|e| self.binary_error(id, e))?
            }

            ConstantWordOffset(_, val) | PcOffsetWord(val) => self
                .binary
                .write_iword_check_size(val as i64)
                .map_err(|e| self.binary_error(id, e))?,

            ConstantByteOffset(_, val) | PcOffsetByte(val) => self
                .binary
                .write_ibyte_check_size(val as i64)
                .map_err(|e| self.binary_error(id, e))?,

            _ => (),
        }

        Ok(())
    }

    fn get_range(&self, pc: i64) -> std::ops::Range<usize> {
        pc as usize..self.get_pc()
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
            IncBin(_file) => {
                panic!()
            },

            Skip(skip) => {
                self.binary.skip(*skip);
            },

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
                        let (arg, id) = self.eval_first_arg(node)?;
                        self.binary
                            .write_byte_check_size(arg)
                            .map_err(|e| self.binary_error(id, e))?;
                    }

                    Extended | Immediate16 => {
                        let (arg, id) = self.eval_first_arg(node)?;

                        self.binary
                            .write_word_check_size(arg)
                            .map_err(|e| self.binary_error(id, e))?;
                    }

                    Relative => {
                        let (arg, id) = self.eval_first_arg(node)?;
                        let val = arg - (pc + ins.size as i64);
                        // offset is from PC after Instruction and operand has been fetched
                        use crate::binary::BinaryError::*;
                        let res = self
                            .binary
                            .write_ibyte_check_size(val)
                            .map_err(|x| match x {
                                DoesNotFit { .. } => self.relative_error(id, val, 8),
                                _ => self.user_error(format!("{:?}", x), node, true),
                            });

                        match &res {
                            Ok(..) => (),
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
                        let (arg, id) = self.eval_first_arg(node)?;
                        let val = arg - (pc + ins.size as i64);
                        // offset is from PC after Instruction and operand has been fetched
                        let res = self.binary.write_iword_check_size(val);

                        res.map_err(|x| match x {
                            DoesNotFit { .. } => self.relative_error(id, val, 16),
                            _ => self.user_error(format!("{:?}", x), node, true),
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

                let range = self.get_range(pc);
                self.source_map
                    .add_mapping(&self.sources, range, pos, ItemType::OpCode);
            }

            ExpandedMacro(mcall) => {
                // We need to tell the source mapper we're expanding a macro so the file / line for
                // everything expanded by the macro will point to the line that instantiated the
                // macro
                let si = self.sources.get_source_info(&mcall.name).unwrap();
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
                        .map_err(|e| {
                            self.binary_error(n.id(), e) })?;
                }

                let range = self.get_range(pc);
                self.source_map
                    .add_mapping(&self.sources, range, pos, ItemType::Command);
            }

            Fcb(..) => {
                for n in node.children() {
                    let x = self.eval_node(n)?;
                    self.binary
                        .write_byte_check_size(x)
                        .map_err(|e| self.binary_error(n.id(), e))?;
                }
                let range = self.get_range(pc);
                self.source_map
                    .add_mapping(&self.sources, range, pos, ItemType::Command);
            }

            Fcc(text) => {
                for c in text.as_bytes() {
                    self.binary
                        .write_byte(*c)
                        .map_err(|e| self.binary_error(id, e))?;
                }
                let range = self.get_range(pc);
                self.source_map
                    .add_mapping(&self.sources, range, pos, ItemType::Command);
            }

            Zmb => {
                let (bytes, _) = self.eval_first_arg(node)?;
                for _ in 0..bytes {
                    self.binary
                        .write_byte(0)
                        .map_err(|e| self.binary_error(id, e))?;
                }
                let range = self.get_range(pc);
                self.source_map
                    .add_mapping(&self.sources, range, pos, ItemType::Command);
            }

            Zmd => {
                let (words, _) = self.eval_first_arg(node)?;
                for _ in 0..words {
                    self.binary
                        .write_word(0)
                        .map_err(|e| self.binary_error(id, e))?;
                }

                let range = self.get_range(pc);
                self.source_map
                    .add_mapping(&self.sources, range, pos, ItemType::Command);
            }

            Fill => {
                let (byte, size) = self.eval_two_args(node)?;

                for _ in 0..size {
                    self.binary
                        .write_ubyte_check_size(byte)
                        .map_err(|e| self.binary_error(id, e))?;
                }

                let range = self.get_range(pc);
                self.source_map
                    .add_mapping(&self.sources, range, pos, ItemType::Command);
            }
            

            Org | AssignmentFromPc(..) | Assignment(..) | Comment(..) | MacroDef(..) | Rmb
            | StructDef(..) => (),

            SetDp => {
                let (dp, _) = self.eval_first_arg(node)?;
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
                        let (v, _) = self.eval_first_arg(node)?;

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
                        let (v, id) = self.eval_first_arg(node)?;

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

    fn size_node(&mut self, mut pc: u64, id: AstNodeId) -> Result<u64, UserError> {
        use crate::util::{ByteSize, ByteSizes};
        use item::Item::*;

        use crate::astformat;
        let x = super::messages::messages();
        let node = self.tree.get(id).unwrap();
        let i = &node.value().item;
        let _old_pc = pc;

        match i {
            Org => {
                let (value, _) = self.eval_first_arg(node)?;
                self.set_item(id, Item::SetPc(value as u16));
                pc = value as u64;
                x.info(format!("Setting put address and org to {pc:04X?}"));
            }

            Put => {
                let x = messages();
                let (value, _) = self.eval_first_arg(node)?;
                let offset = (value - pc as i64) as isize;
                self.set_item(id, Item::SetPutOffset(offset));
                // pc = value as u64;
                x.info(format!("Setting put address {value:04X?}"));
            }

            Rmb => {
                let (bytes, _) = self.eval_first_arg(node)?;

                if bytes < 0 {
                    return Err(self.user_error("Argument for RMB must be positive", node, true));
                };
                
                self.set_item(id, Item::Skip(bytes as usize));

                pc = pc + bytes as u64;
            }

            OpCode(ins, amode) => {
                use emu::isa::AddrModeEnum::*;
                let _line = self
                    .sources
                    .get_source_info(&node.value().pos)
                    .unwrap()
                    .line_str
                    .to_string();

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
                            if let Ok((value, _)) = self.eval_first_arg(node) {
                                let top_byte = ((value >> 8) & 0xff) as u8;

                                if top_byte == dp {
                                    if let Ok(si) = self.sources.get_source_info(&node.value().pos)
                                    {
                                        let x = messages();
                                        x.debug(format!("extended -> direct: {}", si.line_str));
                                    }

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
                            } else {
                                let pos = &node.value().pos;
                                if let Ok(si) = self.sources.get_source_info(pos) {
                                    let x = messages();
                                    x.debug(format!(
                                        "Couldn't eval: {} {} {:?} {} {:X}",
                                        si.line_str, ins.action, pos.src, pos, pc
                                    ));
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
                // println!("{:04X?} {:02X} {line}", old_pc, pc - old_pc);
            }

            AssignmentFromPc(name) => {
                self.symbols
                    .add_symbol_with_value(name, pc as i64)
                    .map_err(|e| {
                        let err = if let SymbolError::Mismatch { expected } = e {
                            format!(
                                "Mismatch symbol {name} : expected {:04X} got : {:04X}",
                                expected, pc
                            )
                        } else {
                            format!("{:?}", e)
                        };
                        self.user_error(err, node, false)
                    })?;
            }

            Block | ExpandedMacro(..) | TokenizedFile(..) => {
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
                let _line = self
                    .sources
                    .get_source_info(&node.value().pos)
                    .unwrap()
                    .line_str
                    .to_string();
                // println!("{:04X?} {:02X} {line}", old_pc, pc - old_pc);
            }

            Zmb => {
                let (v, _) = self.eval_first_arg(node)?;
                assert!(v >= 0);
                pc += v as u64;
            }

            Zmd => {
                let (v, _) = self.eval_first_arg(node)?;
                assert!(v >= 0);
                pc += (v * 2) as u64;
            }

            Fill => {
                let (_, c) = self.eval_two_args(node)?;
                assert!(c >= 0);
                pc += c as u64;
            }

            SetDp => {
                let (dp, _) = self.eval_first_arg(node)?;
                self.set_dp(dp);
            }

            Assignment(..) | Comment(..) | MacroDef(..) | StructDef(..) => (),
            _ => {
                let msg = format!("Unable to size {:?}", i);
                return Err(self.user_error(msg, node, true));
            }
        };

        Ok(pc)
    }
}
