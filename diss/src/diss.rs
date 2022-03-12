use emu::mem::MemReader;
use anyhow::Context;
use clap::{Arg, Command};
use emu::cpu::{IndexModes, IndexedFlags, InstructionDecoder};
use emu::isa::{AddrModeEnum, Dbase, Instruction, InstructionInfo, InstructionType};
use emu::mem::*;
use std::path::PathBuf;

pub struct Disassembly {
    pub text: String,
    pub index_mode: Option<IndexedFlags>,
    pub decoded : InstructionDecoder,
}

pub struct DissCtx {
    pub file: PathBuf,
    pub data: MemBlock<byteorder::BigEndian>,
}

impl DissCtx {
    pub fn from_matches(m: clap::ArgMatches) -> Result<Self, Box<dyn std::error::Error>> {
        use std::fs;
        let file = PathBuf::from(m.value_of("file").unwrap());
        let data: Vec<u8> = fs::read(&file).context("Couldn't read file")?;
        let base_addr = m
            .value_of("base-addr")
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(0);

        let ret = Self {
            file,
            data: MemBlock::from_data(base_addr , "block", &data, true),
        };

        Ok(ret)
    }
}

lazy_static::lazy_static! {
    static ref OPCODES_REC: Dbase = Dbase::new();
}

pub struct Diss<'a> {
    reader: MemReader<'a>,
}

use byteorder::ByteOrder;
use emu::mem::{MemBlock, MemMap};

impl<'a> Diss<'a> {
    pub fn new(mem: &'a mut dyn MemoryIO) -> Self {
        let reader = MemReader::new(mem);
        Diss { reader }
    }

    fn diss_indexed(&mut self) -> (IndexedFlags, String) {
        use emu::cpu::{IndexModes, IndexedFlags};

        let flags = IndexedFlags::new(self.reader.next_byte().unwrap());

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
                let b = self.reader.next_byte().unwrap() as i8;
                format!("{b},{r}")
            }
            IndexModes::RAddi16(r) => {
                let w = self.reader.next_word().unwrap() as i16;
                format!("${w:04X},{r}")
            }

            IndexModes::RAddD(r) => {
                format!("D,{r}")
            }

            IndexModes::PCAddi8 => {
                let b = self.reader.next_byte().unwrap() as i8;
                format!("${b:02X},PC")
            }
            IndexModes::PCAddi16 => {
                let w = self.reader.next_word().unwrap() as i16;
                format!("${w:04X},PC")
            }
            IndexModes::Illegal => "ILLEGAL".to_string(),

            IndexModes::Ea => {
                format!("EA")
            }
        };

        if flags.is_indirect() {
            operand = format!("[{operand}]");
        }

        (flags, operand)
    }

    pub fn diss(&mut self, addr: usize) -> Disassembly {
        let old_addr = self.reader.get_addr();
        self.reader.set_addr(addr);
        let ret = self.diss_next();
        self.reader.set_addr(old_addr);
        ret
    }

    pub fn diss_next(&mut self) -> Disassembly {
        use emu::isa::Instruction;

        let x = emu::cpu::InstructionDecoder::new_from_reader(&mut self.reader).unwrap();

        self.reader.set_addr(x.operand_addr);

        let mut text = format!("{}", x.instruction_info.action);
        let mut index_mode = None;

        use emu::isa::AddrModeEnum::*;

        let operand = match x.instruction_info.addr_mode {
            Indexed => {
                let (flags, text) = self.diss_indexed();
                index_mode = Some(flags);
                text
            }

            Direct => {
                let b = self.reader.next_byte().unwrap();
                format!(">${b:02X}")
            }

            Extended => {
                let w = self.reader.next_word().unwrap();
                format!("${w:04X?}")
            }

            Inherent => {
                format!("")
            }

            Immediate8 => {
                let b = self.reader.next_byte().unwrap();
                format!("#${b:02X}")
            }

            Immediate16 => {
                let w = self.reader.next_word().unwrap();
                format!("#${w:04X?}")
            }

            RegisterSet => {
                let _r = self.reader.next_byte().unwrap();
                format!("RegisterSet SET TBD!")
            }

            RegisterPair => {
                let r = self.reader.next_byte().unwrap();
                let (a,b) = emu::cpu::get_tfr_regs(r);
                format!("{a},{b}")
            }

            Relative => {
                let _b = self.reader.next_byte().unwrap() as i8 as isize;
                let pc = x.addr as isize + _b + 2;
                format!("${pc:04X}")
            }
            Relative16 => {
                let _w = self.reader.next_byte().unwrap() as i16 as isize;
                let pc = x.addr as isize + _w + 2;
                format!("${pc:04X}")
            }
        };

        if !operand.is_empty() {
            text = format!("{} {operand}", text); 
        }

        self.reader.set_addr(x.next_addr);

        Disassembly {
            decoded: x,
            index_mode,
            text,
        }
    }
}
