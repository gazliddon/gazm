use anyhow::Context;
use clap::{Arg, Command};
use emu::cpu::{IndexModes, IndexedFlags, InstructionDecoder};
use emu::isa::{AddrModeEnum, Dbase, Instruction, InstructionInfo, InstructionType};
use emu::mem::MemReader;
use emu::mem::*;
// use serde_json::to_string;
use std::path::PathBuf;

pub struct Disassembly {
    pub text: String,
    pub index_mode: Option<IndexedFlags>,
    pub decoded: InstructionDecoder,
}

pub struct DissCtx {
    pub data: MemBlock<byteorder::BigEndian>,
}

use gazm::numbers::*;

impl DissCtx {
    pub fn from_matches(_m: clap::ArgMatches) -> Result<Self, Box<dyn std::error::Error>> {
        use std::fs;

        let vec = vec![0;0x1_0000];

        let ret = Self {
            data: MemBlock::from_data(0, "mem", &vec, false),
        };

        Ok(ret)
    }
}

lazy_static::lazy_static! {
    static ref OPCODES_REC: Dbase = Dbase::new();
}

#[derive(Default)]
pub struct Diss {}

use byteorder::ByteOrder;
use emu::mem::{MemBlock, MemMap};



struct DissIt<'a> {
    addr: usize,
    diss: &'a Diss,
}

impl<'a> Iterator for DissIt<'a> {
    type Item = Disassembly;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a> DissIt<'a> {
    pub fn new(addr: usize, diss: &'a Diss) -> Self {
        Self { addr, diss }
    }
}

impl Diss {
    pub fn new() -> Self {
        Self::default() 
    }

    fn diss_indexed(&self, reader: &mut MemReader) -> (IndexedFlags, String) {
        use emu::cpu::{IndexModes, IndexedFlags};

        let flags = IndexedFlags::new(reader.next_byte().unwrap());

        let mut operand = match flags.get_index_type() {
            IndexModes::ROff(r, off) => {
                format!("${off:02X},{r}")
            }
            IndexModes::RPlus(r) => {
                format!(",{r}+")
            }
            IndexModes::RPlusPlus(r) => {
                format!(",{r}++")
            }
            IndexModes::RSub(r) => {
                format!(",-{r}")
            }
            IndexModes::RSubSub(r) => {
                format!(",--{r}")
            }
            IndexModes::RZero(r) => {
                format!(",{r}")
            }
            IndexModes::RAddB(r) => {
                format!("B,{r}")
            }
            IndexModes::RAddA(r) => {
                format!("A,{r}")
            }
            IndexModes::RAddi8(r) => {
                let b = reader.next_byte().unwrap() as i8;
                format!("{b},{r}")
            }
            IndexModes::RAddi16(r) => {
                let w = reader.next_word().unwrap() as i16;
                format!("${w:04X},{r}")
            }

            IndexModes::RAddD(r) => {
                format!("D,{r}")
            }

            IndexModes::PCAddi8 => {
                let b = reader.next_byte().unwrap() as i8;
                format!("${b:02X},PC")
            }
            IndexModes::PCAddi16 => {
                let w = reader.next_word().unwrap() as i16;
                format!("${w:04X},PC")
            }
            IndexModes::Illegal => "ILLEGAL".to_string(),

            IndexModes::Ea => {
                "EA".to_string()
            }
        };

        if flags.is_indirect() {
            operand = format!("[{operand}]");
        }

        (flags, operand)
    }

    pub fn diss(&self, mem: &mut dyn MemoryIO, addr: usize) -> Disassembly {
        use emu::isa::Instruction;
        let mut reader = MemReader::new(mem);
        reader.set_addr(addr);

        let x = emu::cpu::InstructionDecoder::new_from_reader(&mut reader).unwrap();

        reader.set_addr(x.operand_addr);

        let mut text = x.instruction_info.action.to_string();
        let mut index_mode = None;

        use emu::isa::AddrModeEnum::*;

        let operand = match x.instruction_info.addr_mode {
            Indexed => {
                let (flags, text) = self.diss_indexed(&mut reader);
                index_mode = Some(flags);
                text
            }

            Direct => {
                let b = reader.next_byte().unwrap();
                format!(">${b:02X}")
            }

            Extended => {
                let w = reader.next_word().unwrap();
                format!("${w:04X?}")
            }

            Inherent => {
                "".to_string()
            }

            Immediate8 => {
                let b = reader.next_byte().unwrap();
                format!("#${b:02X}")
            }

            Immediate16 => {
                let w = reader.next_word().unwrap();
                format!("#${w:04X?}")
            }

            RegisterSet => {
                let _r = reader.next_byte().unwrap();
                "RegisterSet SET TBD!".to_owned()
            }

            RegisterPair => {
                let r = reader.next_byte().unwrap();
                let (a, b) = emu::cpu::get_tfr_regs(r);
                format!("{a},{b}")
            }

            Relative => {
                let _b = reader.next_byte().unwrap() as i8 as isize;
                let pc = x.addr as isize + _b + 2;
                format!("${pc:04X}")
            }
            Relative16 => {
                let _w = reader.next_byte().unwrap() as i16 as isize;
                let pc = x.addr as isize + _w + 2;
                format!("${pc:04X}")
            }
        };

        if !operand.is_empty() {
            text = format!("{} {operand}", text);
        }

        Disassembly {
            decoded: x,
            index_mode,
            text,
        }
    }
}
