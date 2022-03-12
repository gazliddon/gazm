use crate::memreader::MemReader;
use anyhow::Context;
use clap::{Arg, Command};
use emu::cpu::{IndexModes, IndexedFlags};
use emu::isa::{AddrModeEnum, Dbase, Instruction, InstructionInfo, InstructionType};
use emu::mem::*;
use std::path::PathBuf;

pub struct Disassembly {
    pub text: String,
    pub ins: Instruction,
    pub addr: usize,
    pub size: usize,
    pub cycles: usize,
    pub index_mode: Option<IndexedFlags>,
    pub bytes: Vec<u8>,
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

pub struct Diss<'a, M: MemoryIO> {
    reader: MemReader<'a, M>,
}

use byteorder::ByteOrder;
use emu::mem::{MemBlock, MemMap};

impl<'a, M: MemoryIO> Diss<'a, M> {
    pub fn new(mem: &'a mut M) -> Self {
        let reader = MemReader::new(mem);
        Diss { reader }
    }

    fn diss_indexed(&mut self) -> (IndexedFlags, String, usize) {
        let cycles = 0;
        let x = self.reader.next_byte().unwrap();
        use emu::cpu::{IndexModes, IndexedFlags};

        let flags = IndexedFlags::new(x);

        let mut operand = match flags.get_index_type() {
            IndexModes::ROff(r, off) => {
                format!("{off},{r}")
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
                format!("{w},{r}")
            }

            IndexModes::RAddD(r) => {
                format!("D,{r}")
            }

            IndexModes::PCAddi8 => {
                let b = self.reader.next_byte().unwrap() as i8;
                format!("{b},PC")
            }
            IndexModes::PCAddi16 => {
                let w = self.reader.next_word().unwrap() as i16;
                format!("{w},PC")
            }
            IndexModes::Illegal => "???".to_string(),

            IndexModes::Ea => {
                panic!()
            }
        };

        if flags.is_indirect() {
            operand = format!("[{operand}]");
        }

        (flags, operand, cycles)
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
        let addr = self.reader.get_addr();

        let x = self.reader.peek_word().unwrap();

        let opcode = if x > 0xff {
            self.reader.next_word().unwrap()
        } else {
            self.reader.next_byte().unwrap() as u16
        };

        let mut index_mode = None;

        let ins = OPCODES_REC.get(opcode).clone();
        let mut cycles = ins.cycles as usize;

        use emu::isa::AddrModeEnum::*;

        let operand = match ins.addr_mode {
            Indexed => {
                let (flags, text, icycles) = self.diss_indexed();
                index_mode = Some(flags);
                cycles += icycles;
                text
            }

            Direct => {
                let b = self.reader.next_byte().unwrap();
                format!(">{b}")
            }

            Extended => {
                let w = self.reader.next_word().unwrap();
                format!("{w}")
            }

            Inherent => {
                format!("")
            }

            Immediate8 => {
                let b = self.reader.next_byte().unwrap();
                format!("#{b}")
            }

            Immediate16 => {
                let w = self.reader.next_word().unwrap();
                format!("#{w}")
            }

            RegisterSet => {
                let _r = self.reader.next_byte().unwrap();
                format!("RegisterSet SET TBD!")
            }

            RegisterPair => {
                let _r = self.reader.next_byte().unwrap();
                format!("RegisterPair TBD!")
            }

            Relative => {
                let _b = self.reader.next_byte().unwrap();
                format!("Relative TBD!")
            }
            Relative16 => {
                let _w = self.reader.next_byte().unwrap();
                format!("Relative16 TBD")
            }
        };

        let text = if operand.is_empty() {
            format!("{}", ins.action)
        } else {
            format!("{} {operand}", ins.action)
        };

        let r = addr..self.reader.get_addr();
        let size = r.len();
        let bytes = r
            .clone()
            .map(|i| {
                self.reader
                    .get_mem()
                    .inspect_byte(i.try_into().unwrap())
                    .unwrap()
            })
            .collect();

        Disassembly {
            ins,
            addr,
            index_mode,
            cycles,
            text,
            bytes,
            size,
        }
    }
}
