#![allow(unused_imports)]
#![allow(dead_code)]
mod commands;
mod diss;
mod term;
use emu::{cpu::CpuErr, mem::MemBlock};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
enum ComputerErr {
    #[error(transparent)]
    Cpu(#[from] CpuErr),
}

type CResult<T> = Result<T, ComputerErr>;

struct RunResult {
    next_pc: usize,
    cycles: usize,
}

trait Computer {
    fn step(&mut self) -> CResult<RunResult>;
    fn step_over(&mut self) -> CResult<RunResult>;
    fn run_cycles(&mut self, cycles: usize) -> CResult<RunResult>;
    fn run_instructions(&mut self, instructions: usize) -> CResult<RunResult>;
    fn get_pc(&self) -> usize;
    fn mem_mut(&mut self) -> &mut dyn MemoryIO;
    fn mem(&self) -> &dyn MemoryIO;
    fn reset(&mut self);
}

struct Stargate {
    pub regs: emu::cpu::Regs,
    pub mem: emu::mem::MemBlock<BigEndian>,
    pub pins: emu::cpu::Pins,
}

fn load_binary_file<P: AsRef<std::path::Path>>(filename: P) -> Vec<u8> {
    use std::fs::File;
    use std::io::Read;
    let mut f = File::open(&filename).expect("no file found");
    let metadata = std::fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    buffer
}

impl Stargate {
    pub fn new() -> Self {
        let mem = emu::mem::MemBlock::new("mem", false, &(0..0x1_0000));
        Self {
            regs: Default::default(),
            mem,
            pins: Default::default(),
        }
    }

    fn make_ctx(&mut self) -> cpu::Context {
        cpu::Context::new(&mut self.mem, &mut self.regs, &mut self.pins).unwrap()
    }
}

impl Computer for Stargate {
    fn step(&mut self) -> CResult<RunResult> {
        let mut ctx = self.make_ctx();

        ctx.step()?;

        Ok(RunResult {
            next_pc: ctx.get_pc(),
            cycles: ctx.cycles(),
        })
    }

    fn reset(&mut self) {
        use emu::cpu::{Flags, Regs};

        let pc = self.mem.load_word(0xfffe).unwrap();

        self.regs = Regs {
            pc,
            flags: Flags::I | Flags::F,
            ..Default::default()
        };
    }

    fn step_over(&mut self) -> CResult<RunResult> {
        panic!()
    }

    fn run_cycles(&mut self, cycles: usize) -> CResult<RunResult> {
        let mut ret = RunResult {
            next_pc: self.get_pc(),
            cycles: 0,
        };

        while ret.cycles <= cycles {
            let r = self.step()?;
            ret.cycles += r.cycles;
            ret.next_pc = r.next_pc;
        }

        Ok(ret)
    }

    fn run_instructions(&mut self, instructions: usize) -> CResult<RunResult> {
        let mut ret = RunResult {
            next_pc: self.get_pc(),
            cycles: 0,
        };

        for _i in 0..instructions {
            let r = self.step()?;
            ret.cycles += r.cycles;
            ret.next_pc = r.next_pc;
        }

        Ok(ret)
    }

    fn get_pc(&self) -> usize {
        self.regs.pc as usize
    }

    fn mem_mut(&mut self) -> &mut dyn MemoryIO {
        &mut self.mem
    }

    fn mem(&self) -> &dyn MemoryIO {
        &self.mem
    }
}

// use anyhow::Context;

use byteorder::BigEndian;
use emu::{
    cpu,
    mem::{MemReader, MemoryIO},
};

use emu::utils::rle::Run;
use gazm::{parse::commands::parse_command, parse::numbers::*};
use nom_locate::LocatedSpan;

pub fn parse() -> clap::ArgMatches {
    use clap::{Arg, Command};
    Command::new("diss")
        .about("6809 diss")
        .author("gazaxian")
        .version("0.1")
        .get_matches()
}

fn mk_diss_it(
    data: &mut dyn MemoryIO,
    addr: usize,
) -> impl Iterator<Item = diss::Disassembly> + '_ {
    let mut addr = addr;
    std::iter::from_fn(move || {
        let diss = diss::Diss::new();
        let x = diss.diss(data, addr);
        addr = x.decoded.next_addr;
        Some(x)
    })
}

fn to_hex_str(mem: &[u8]) -> String {
    let v: Vec<_> = mem.iter().map(|b| format!("{b:02X}")).collect();
    v.join(" ")
}

struct DebugCtx {
    default_hex: bool,
    last_command: Option<commands::Command>,
    current_addr: usize,
}

impl DebugCtx {
    pub fn new() -> Self {
        Self {
            default_hex: false,
            last_command: None,
            current_addr: 0,
        }
    }
}

fn do_command(dbg_ctx: &mut DebugCtx, text: &str, x: commands::Command, sg: &mut Stargate) {
    use commands::Command;

    match &x {
        Command::LoadBin(file, addr) => {
            println!(
                "Not implemented : load bin file {} to 0x{addr:04x}",
                file.to_string_lossy()
            )
        }

        Command::LoadSym(file) => {
            println!("Loading {}", file.to_string_lossy());

            let sd = emu::utils::sources::SourceDatabase::from_json(file);

            match sd {
                Ok(sd) => {
                    for bin in &sd.bin_written {
                        let data = load_binary_file(&bin.file);
                        sg.mem_mut()
                            .upload(bin.addr.start, &data)
                            .expect("Can't upload rom file");
                        println!(
                            "Loading: {}: 0x{:X} bytes to 0x{:04X}",
                            bin.file.file_name().unwrap().to_string_lossy(),
                            bin.addr.len(),
                            bin.addr.start
                        );
                    }
                    sg.reset();
                    println!("{}", sg.regs);
                }
                Err(e) => {
                    println!("{e}")
                }
            }
        }

        Command::Step => {
            let x = diss::Diss::new();
            let d = x.diss(&mut sg.mem, sg.regs.pc as usize);
            let i = sg.step().unwrap();
            sg.regs.pc = i.next_pc as u16;
            println!("{}", d.text);
        }

        Command::Reset => {
            sg.reset();
            println!("{}", sg.regs);
        }

        Command::Regs => {
            println!("{}", sg.regs);
        }

        Command::Hex => {
            println!("Default radix : Hex");
            dbg_ctx.default_hex = true;
        }
        Command::Dec => {
            println!("Default radix : Decimal");
            dbg_ctx.default_hex = false;
        }

        Command::Diss(d_addr) => {
            let mut addr = d_addr.unwrap_or(dbg_ctx.current_addr as isize);

            println!("Disassembling {addr:04X}");
            let mut i = mk_diss_it(sg.mem_mut(), addr as usize);

            for _ in 0..10 {
                if let Some(ins) = i.next() {
                    let mem = to_hex_str(&ins.decoded.data);
                    println!(" {:04X}  {mem:15} {}", addr, ins.text);
                    addr = ins.decoded.next_addr as isize;
                } else {
                    break;
                }
            }
            dbg_ctx.current_addr = addr as usize;
        }

        Command::Mem(d_addr) => {
            let addr = d_addr.unwrap_or(dbg_ctx.current_addr as isize);
            let mut i = MemReader::new(&mut sg.mem);
            i.set_addr(addr as usize);

            for _ in 0..8 {
                print!(" {:04X} ", i.get_addr());
                for _ in 0..8 {
                    if let Ok(b) = i.next_byte() {
                        print!("{b:02X} ");
                    } else {
                        print!("?? ");
                    }
                }
                println!();
            }

            dbg_ctx.current_addr = i.get_addr();
        }

        _ => {
            println!("Unexpected command {text}")
        }
    }

    dbg_ctx.last_command = Some(x);
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    use term::{Term, TermOutput};

    let mut term = Term::new();

    let mut running = true;

    while running {
        let output = term.flush();

        for y in output {
            match &y {
                TermOutput::Text(x) => println!("got {x:?}"),
                _ => {
                    running = false;
                    break;
                }
            };
        }
    }

    term.quit();

    Ok(())
}
