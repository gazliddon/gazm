use crate::cpu::{ Regs, Flags };

use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

use regex::Regex;

use crate::mem::{MemoryIO};
use crate::diss::Disassembler;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct StepError {
    pub regs         : bool,
    pub disassembly  : bool,
    pub mem          : bool,
    pub cycles: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Step {
    pub regs         : Regs,
    pub disassembly  : Option<String>,
    pub mem          : Option<[ u8; 5]>,
    pub cycles       : usize,
    pub digest       : Option<String>,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let mem_str = match self.mem {
            Some(m) =>
                format!("{:02x} {:02x} {:02x} {:02x} {:02x}", m[0], m[1], m[2], m[3], m[4]),
            _ => "?? ?? ?? ?? ??".to_string()
        };
            

        let diss = match self.disassembly.clone() {
            Some(t) => t,
            _ => "NO DISS".to_string()
        };

        write!(f,
               "{} {:16} {} {:>8}",
               self.regs,
               diss,
               mem_str,
               self.cycles)
    }
}

impl Step {
    pub fn compare(&self, other : &Self) -> StepError {

        StepError {
            regs         : self.regs == other.regs,
            disassembly  : self.disassembly == other.disassembly,
            mem          : self.mem == other.mem,
            cycles       : self.cycles == other.cycles,
        }
    }

    fn grab_mem<M : MemoryIO>(&mut self, _mem : &M, _addr : u16) {
        if self.mem.is_some() {
        }

    }

    pub fn from_sim<M : MemoryIO>(mem : &mut M, regs : &Regs, cycles : usize) -> Step {

        let mut diss = Disassembler::new();
        let (_, txt) =  diss.diss(mem, regs.pc, None);

        let mut step = Step {
            regs          : regs.clone(),
            disassembly   : Some(txt),
            cycles,
            .. Default::default()
        };

        step.grab_mem(mem, regs.pc);

        step
    }

    pub fn from_string(text :&str) -> Result<Step, String> {

        lazy_static!{
            static ref RE : Regex =
                Regex::new(r"(?x)^
                (?P<pc>[[:xdigit:]]{4})\s
                (?P<d>[a-f0-9]{4})\s
                (?P<x>[a-f0-9]{4})\s
                (?P<y>[a-f0-9]{4})\s
                (?P<u>[a-f0-9]{4})\s
                (?P<s>[a-f0-9]{4})\s
                (?P<dp>[a-f0-9]{2})\s
                (?P<flags>[0-1]{8})\s
                (?P<diss>.*?)
                \s+
                (?P<m0>[[:xdigit:]]{2})\s
                (?P<m1>[[:xdigit:]]{2})\s
                (?P<m2>[[:xdigit:]]{2})\s
                (?P<m3>[[:xdigit:]]{2})\s
                (?P<m4>[[:xdigit:]]{2})\s+
                (?P<cycles>\d+)\s$").unwrap();
        }

        assert!(RE.is_match(text));

        let captures = RE.captures(text).unwrap();

        let as_u8_from_bin = |i:&str| u32::from_str_radix(&captures[i], 2).unwrap() as u8;
        let as_u8 = |i:&str| u32::from_str_radix(&captures[i], 16).unwrap() as u8;
        let as_u16 = |i:&str| u32::from_str_radix(&captures[i], 16).unwrap() as u16;
        let as_usize = |i:&str| usize::from_str_radix(&captures[i], 10).unwrap();
        let as_string = |i:&str| captures[i].to_string();

        let mut regs = Regs {
            pc   : as_u16("pc"),
            a    : 0,
            b    : 0,
            x    : as_u16("x"),
            y    : as_u16("y"),
            s    : as_u16("s"),
            u    : as_u16("u"),
            dp   : as_u8("dp"),
            flags: Flags::new(as_u8_from_bin("flags"))
        };

        // println!("{}", as_string("m0"));

        let d = as_u16("d");

        regs.set_d(d);

        let r = Step {
            regs,
            disassembly   : Some(as_string("diss")),
            mem           : Some([ as_u8("m0"), as_u8("m1"), as_u8("m2"), as_u8("m3"), as_u8("m4"), ]),
            cycles : as_usize("cycles"),
            .. Default::default()
        };

        Ok(r)
    }

    fn to_string(&self) -> String {
        panic!("fucked!")
    }
}

pub fn read_step_log( file_name : &str) -> Vec<Step> {

    let f = File::open(file_name).unwrap();

    BufReader::new(&f)
        .lines()
        .filter_map(|l| l.ok())
        .filter_map(|l| Step::from_string(&l).ok())
        .collect()
}

