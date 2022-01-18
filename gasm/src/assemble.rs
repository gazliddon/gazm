use colored::*;
use emu::cpu::RegEnum;
use nom::combinator::recognize;

use crate::ast::AstNodeRef;
use crate::ast::AstTree;
use crate::ast::{Ast, AstNodeId, AstNodeMut};
use crate::cli;
use crate::cli::Context;
use crate::error::UserError;
use crate::eval;
use crate::eval::eval;
use crate::item;
use crate::item::AddrModeParseType;
use crate::item::IndexParseType;
use crate::symbols::SymbolTable;
use crate::util;
use crate::util::info;
use crate::util::ByteSize;
use item::{Item, Node};
use romloader::ResultExt;
use std::collections::HashSet;
use std::path::PathBuf;
use std::vec;

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
        registers = registers | 0x01;
    }

    if regs.contains(&A) {
        registers = registers | 0x02;
    }
    if regs.contains(&B) {
        registers = registers | 0x04;
    }

    if regs.contains(&DP) {
        registers = registers | 0x08;
    }

    if regs.contains(&X) {
        registers = registers | 0x10;
    }

    if regs.contains(&Y) {
        registers = registers | 0x20;
    }

    if regs.contains(&U) || regs.contains(&S) {
        registers = registers | 0x40;
    }

    if regs.contains(&PC) {
        registers = registers | 0x80;
    }
    registers
}

pub struct Binary {
    write_address: usize,
    written: bool,
    range: Option<(usize, usize)>,
    data: Vec<u8>,
}

impl Default for Binary {
    fn default() -> Self {
        Self {
            write_address: 0,
            written: false,
            range: None,
            data: vec![0; 0x10000],
        }
    }
}

impl Binary {
    fn dirty(&mut self) {
        self.written = true;
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn bump_write_address(&mut self, n: usize) {
        self.write_address += n;
    }

    pub fn get_write_address(&self) -> usize {
        self.write_address
    }

    pub fn set_write_address(&mut self, pc: usize) {
        self.write_address = pc
    }

    pub fn set_write_addr(&mut self, pc: usize) {
        self.write_address = pc;
    }

    pub fn get_range(&self) -> Option<(usize, usize)> {
        self.range
    }

    pub fn write_byte_check_size(&mut self, val: i64) -> Result<(), ()> {
        let bits = val >> 8;
        if bits == -1 || bits == 0 {
            self.write_byte(val as u8);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn write_word_check_size(&mut self, val: i64) -> Result<(), ()> {
        let bits = val >> 16;
        if bits == -1 || bits == 0 {
            self.write_word(val as u16);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn write_byte(&mut self, val: u8) {
        let pc = self.write_address;

        if let Some((mut low, mut high)) = self.range {
            if pc < low {
                low = pc
            }

            if pc > high {
                high = pc
            }

            self.range = Some((low, high))
        } else {
            self.range = Some((pc, pc))
        }
        self.data[pc] = val;

        self.write_address += 1;
    }
    pub fn fill(&mut self, count: usize, byte: u8) {
        for _i in 0..count {
            self.write_byte(byte)
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.write_byte(*b)
        }
    }

    pub fn get_bytes(&self, pc: usize, count: usize) -> &[u8] {
        &self.data[pc..(pc + count)]
    }

    pub fn write_word(&mut self, val: u16) {
        self.write_byte((val >> 8) as u8);
        self.write_byte((val & 0xff) as u8);
    }
}

pub struct Assembler {
    symbols: crate::symbols::SymbolTable,
    sources: crate::sourcefile::Sources,
    bin: Binary,
    tree: crate::ast::AstTree,
}

fn eval_node(
    symbols: &SymbolTable,
    node: AstNodeRef,
    sources: &crate::sourcefile::Sources,
) -> Result<i64, UserError> {
    eval(&symbols, node).map_err(|err| {
        let info = sources.get_source_info_from_value(node.value()).unwrap();
        UserError::from_ast_error(err, &info)
    })
}

impl From<crate::ast::Ast> for Assembler {
    fn from(ast: crate::ast::Ast) -> Self {
        Self {
            symbols: ast.symbols,
            sources: ast.sources,
            bin: Binary::new(),
            tree: ast.tree,
        }
    }
}

impl Assembler {
    pub fn assemble_indexed_opcode(
        &mut self,
        _ins: &emu::isa::Instruction,
        _addr_mode: &AddrModeParseType,
        _node: AstNodeRef,
    ) -> Result<(), UserError> {
        todo!("assemble indexed opcode")
    }

    fn write_byte_check_size(&mut self, n: AstNodeId, val: i64) -> Result<(), UserError> {
        self.bin.write_byte_check_size(val).map_err(|_| {
            let n = self.tree.get(n).unwrap();
            let info = &self.sources.get_source_info_from_value(n.value()).unwrap();
            let msg = format!("{:4X} does not fit in a byte", val);
            UserError::from_text(msg, info, &n.value().pos)
        })
    }

    fn write_word_check_size(&mut self, n: AstNodeId, val: i64) -> Result<(), UserError> {
        self.bin.write_word_check_size(val).map_err(|_| {
            let n = self.tree.get(n).unwrap();
            let info = &self.sources.get_source_info_from_value(n.value()).unwrap();
            let msg = format!("{:4X} does not fit in a word", val);
            UserError::from_text(msg, info, &n.value().pos)
        })
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

    fn user_error<S: Into<String>>(&self, err: S, node: AstNodeRef) -> UserError {
        let info = self
            .sources
            .get_source_info_from_value(node.value())
            .unwrap();
        UserError::from_text(err, &info, &node.value().pos)
    }

    fn eval_node(&self, node: AstNodeRef) -> Result<i64, UserError> {
        eval(&self.symbols, node).map_err(|err| {
            let info = self
                .sources
                .get_source_info_from_value(node.value())
                .unwrap();
            UserError::from_ast_error(err, &info)
        })
    }

    fn eval_first_arg(&self, node: AstNodeRef) -> Result<(i64, AstNodeId), UserError> {
        let c = node
            .first_child()
            .ok_or(self.user_error("Missing argument", node))?;
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

    pub fn assemble(&mut self) -> Result<(), UserError> {
        info("Assembling...", |_| {
            self.assemble_node(self.tree.root().id())
        })
    }

    fn assemble_indexed(&mut self, id: AstNodeId, imode: IndexParseType) -> Result<(), UserError> {
        let idx_byte = imode.get_index_byte();
        self.bin.write_byte(idx_byte);
        use item::IndexParseType::*;
        let node = self.tree.get(id).unwrap();

        match imode {
            PCOffset(..) | ConstantOffset(..) => {
                panic!("Should not happen")
            }

            ExtendedIndirect => {
                let (val, _) = self.eval_first_arg(node)?;
                self.write_word_check_size(id, val)?
            }

            ConstantWordOffset(_, val, _) | PcOffsetWord(val, _) => {
                self.write_word_check_size(id, val as i64)?;
            }

            ConstantByteOffset(_, val, _) | PcOffsetByte(val, _) => {
                self.write_byte_check_size(id, val as i64)?;
            }

            _ => (),
        }

        Ok(())
    }

    fn assemble_node(&mut self, id: AstNodeId) -> Result<(), UserError> {
        let x = super::messages::messages();

        use item::Item::*;

        let node = self.tree.get(id).unwrap();
        let frag = self
            .sources
            .get_source_info_from_value(&node.value())
            .unwrap()
            .fragment
            .to_string();

        // let node = self.tree.get(id).unwrap();
        let i = &node.value().item.clone();
        let pc = self.bin.get_write_address() as i64;

        match i {
            SetPc(pc) => {
                self.bin.set_write_addr(*pc as usize);
                x.debug(format!("Set PC to {:02X}", *pc));
            }

            OpCode(ins, amode) => {
                use emu::isa::AddrModeEnum::*;
                let ins_amode = ins.addr_mode;

                if ins.opcode > 0xff {
                    self.bin.write_word(ins.opcode);
                } else {
                    self.bin.write_byte(ins.opcode as u8);
                }

                match ins_amode {
                    Indexed => {
                        if let AddrModeParseType::Indexed(imode) = amode {
                            self.assemble_indexed(id, imode.clone())?;
                        }
                    }

                    Immediate8 | Direct => {
                        let (arg, id) = self.eval_first_arg(node)?;
                        self.write_byte_check_size(id, arg)?;
                    }

                    Immediate16 | Extended => {
                        let (arg, id) = self.eval_first_arg(node)?;
                        self.write_word_check_size(id, arg)?;
                    }

                    Relative => {
                        let (arg, id) = self.eval_first_arg(node)?;
                        // offset is from PC after Instruction and operand has been fetched
                        self.write_byte_check_size(id, arg - ( pc + ins.size as i64 ))?;
                    }

                    Relative16 => {
                        let (arg, id) = self.eval_first_arg(node)?;
                        // offset is from PC after Instruction and operand has been fetched
                        self.write_word_check_size(id,  arg - ( pc  + ins.size as i64 ))?;
                    }

                    Inherent => {}

                    RegisterPair => {
                        if let AddrModeParseType::RegisterPair(a, b) = amode {
                            self.bin.write_byte(reg_pair_to_flags(*a, *b));
                        } else {
                            panic!("Whut!")
                        }
                    }

                    RegisterSet => {
                        // println!("{:#?}", node);
                        
                        let rset = &node.first_child().unwrap().value().item;
                        if let Item::RegisterSet(regs) = rset {
                            self.bin.write_byte(registers_to_flags(regs));
                        } else {
                            panic!("Whut!")
                        }
                    }
                };

                let pc = pc as usize;
                let written = self.bin.get_write_address() - pc;
                let bytes = self.bin.get_bytes(pc as usize, written);
                let bytes_str: Vec<_> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                let bytes_str = bytes_str.join("");
                let msg = format!("{:04X}  {:20} {}", pc, bytes_str, frag);

                if ins_amode == Indexed {
                    if let AddrModeParseType::Indexed(imode) = amode {
                        let msg = format!("{:50} {:?}", msg, imode);
                        x.info(msg);
                    }
                } else {
                    x.success(msg);
                }
            }

            TokenizedFile(..) => {
                let children: Vec<_> = node.children().map(|n| n.id()).collect();
                for c in children {
                    self.assemble_node(c)?;
                }
            }

            Fdb(_) => {
                for n in node.children() {
                    let x = self.eval_node(n)?;
                    self.bin
                        .write_word_check_size(x)
                        .map_err(|_| self.user_error("Does not fit in a word", n))?;
                }
            }

            Fcb(_) => {
                for n in node.children() {
                    let x = self.eval_node(n)?;
                    self.bin
                        .write_byte_check_size(x)
                        .map_err(|_| self.user_error("Does not fit in a word", n))?;
                }
            }

            Zmb => {
                let (bytes, _) = self.eval_first_arg(node)?;
                for _ in 0..bytes {
                    self.bin.write_byte(0)
                }
            }

            Zmd => {
                let (words, _) = self.eval_first_arg(node)?;
                for _ in 0..words {
                    self.bin.write_word(0)
                }
            }

            Fill => {
                let (byte, size) = self.eval_two_args(node)?;
                for _ in 0..size {
                    self.bin
                        .write_byte_check_size(byte)
                        .map_err(|_| self.user_error("Does not fit in a word", node))?;
                }
            }

            Org | AssignmentFromPc(..) | Assignment(..) | Comment(..) => (),

            _ => {
                println!("Unable to assemble {:?}", i);
            }
        }

        Ok(())
    }

    fn size_node(&mut self, mut pc: u64, id: AstNodeId) -> Result<u64, UserError> {
        use crate::util::{ByteSize, ByteSizes};
        use item::Item::*;

        use crate::astformat;
        let x = super::messages::messages();
        let node = self.tree.get(id).unwrap();
        let i = &node.value().item;

        match i {
            Org => {
                let (value, _) = self.eval_first_arg(node)?;
                self.set_item(id, Item::SetPc(value as u16));
                pc = value as u64;
            }

            OpCode(ins, amode) => {
                use emu::isa::AddrModeEnum::*;

                pc = pc + ins.size as u64;

                if let AddrModeParseType::Indexed(pmode) = amode {
                    use item::IndexParseType::*;

                    match pmode {
                        ConstantByteOffset(..)
                        | PcOffsetByte(..)
                        | PcOffsetWord(..)
                        | ConstantWordOffset(..)
                        | Constant5BitOffset(..) => {
                            panic!()
                        }

                        ConstantOffset(r, indirect) => {
                            let (v, _) = self.eval_first_arg(node)?;

                            let mut bs = v.byte_size();

                            if let ByteSizes::Bits5(..) = bs {
                                if *indirect {
                                    bs.promote();
                                }
                            }

                            let new_amode = match bs {
                                ByteSizes::Bits5(v) => Constant5BitOffset(*r, v, *indirect),
                                ByteSizes::Word(v) => {
                                    pc += 2;
                                    ConstantWordOffset(*r, v, *indirect)
                                }
                                ByteSizes::Byte(v) => {
                                    pc += 1;
                                    ConstantByteOffset(*r, v, *indirect)
                                }
                            };

                            let ins = ins.clone();
                            self.set_item(id, OpCode(ins, AddrModeParseType::Indexed(new_amode)));
                        }

                        PCOffset(indirect) => {
                            let (v, id) = self.eval_first_arg(node)?;

                            let new_amode = match v.byte_size() {
                                ByteSizes::Bits5(v) | ByteSizes::Byte(v) => {
                                    pc = pc + 1;
                                    PcOffsetByte(v, *indirect)
                                }
                                ByteSizes::Word(v) => {
                                    pc = pc + 2;
                                    PcOffsetWord(v, *indirect)
                                }
                            };

                            let ins = ins.clone();
                            self.set_item(id, OpCode(ins, AddrModeParseType::Indexed(new_amode)));
                        }

                        ExtendedIndirect => pc += 2,
                        _ => (),
                    };
                }
            }

            AssignmentFromPc(name) => {
                let msg = format!("{} -> ${:04X}", name, pc);
                x.debug(msg);
                self.symbols
                    .add_symbol_with_value(name, pc as i64, node.id())
                    .unwrap();
            }

            TokenizedFile(..) => {
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

            Zmb => {
                let (v, _) = self.eval_first_arg(node)?;
                assert!(v >= 0);
                pc = pc + v as u64;
            }

            Zmd => {
                let (v, _) = self.eval_first_arg(node)?;
                assert!(v >= 0);
                pc = pc + (v * 2) as u64;
            }

            Fill => {
                let (_, c) = self.eval_two_args(node)?;
                assert!(c >= 0);
                pc = pc + c as u64;
            }

            Assignment(..) | Comment(..) => (),

            _ => {
                println!("Unable to size {:?}", i);
            }
        };

        Ok(pc)
    }
}
