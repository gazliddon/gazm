use super::mem::MemoryIO;

use romloader::{ Dbase, Instruction };
use std::str;

const RBYTE : &[u8] = include_bytes!("resources/opcodes.json");

lazy_static! {
    static ref DBASE: Dbase = {
        let bytes_str = str::from_utf8(RBYTE).unwrap();
        Dbase::from_text(bytes_str)
    };
}

#[derive(Debug, Clone)]
pub struct InstructionDecoder {
    index: u16,
    pub op_code: u16,
    pub cycles: u16,
    pub addr: u16,
    pub data: [u8; 5],
    pub next_addr: u16,
    pub instruction_info: &'static Instruction,
}

// Decode an op
// Takes memory read as closure
// means we can destructively read op code when emulating
// or non destructively inspect for disassembly

fn decode_op(addr: u16, mut read: impl FnMut(u16) -> u8) -> InstructionDecoder {
    // Array insrtuction bytes is copied to
    let mut data = [0, 0, 0, 0, 0];
    // fetch index
    let mut index = 0;

    // fetch a byte from the memory reader
    // store it i the instruction array
    // and bump the read index
    let mut fetch = || {
        let b = read(addr.wrapping_add(index));
        data[index as usize] = b;
        index += 1;
        b as u16
    };

    // get the first byte of the opcode
    let a = fetch();

    // Fetch the next byte if it's an extended opcode
    let op_code = match a {
        0x10 | 0x11 => (a << 8) + fetch(),
        _ => a,
    };

    // Fetch a reference to the extended infomation about this
    // opcode
    let instruction_info = DBASE.get(op_code);

    // Create the decoded instruction
    InstructionDecoder {
        next_addr: addr.wrapping_add(instruction_info.size),
        index,
        addr,
        op_code,
        instruction_info,
        cycles: instruction_info.cycles,
        data,
    }
}

impl InstructionDecoder {
    pub fn new(_addr: u16) -> Self {
        panic!()
    }

    pub fn fetch_inspect_word(&mut self, mem: &dyn MemoryIO) -> u16 {
        let w = mem.inspect_word(self.addr.wrapping_add(self.index));
        self.index = self.index.wrapping_add(2);
        w
    }

    pub fn fetch_inspecte_byte(&mut self, mem: &dyn MemoryIO) -> u8 {
        let b = mem.inspect_byte(self.addr.wrapping_add(self.index));
        self.index = self.index.wrapping_add(1);
        b
    }

    pub fn new_from_inspect_mem(addr: u16, mem: &dyn MemoryIO) -> Self {
        decode_op(addr, |addr| mem.inspect_byte(addr))
    }

    pub fn new_from_read_mem(addr: u16, mem: &mut dyn MemoryIO) -> Self {
        decode_op(addr, |addr| mem.load_byte(addr))
    }

    pub fn fetch_byte(&mut self, mem: &mut dyn MemoryIO) -> u8 {
        let b = mem.load_byte(self.addr.wrapping_add(self.index));
        self.index = self.index.wrapping_add(1);
        b
    }

    pub fn fetch_word(&mut self, mem: &mut dyn MemoryIO) -> u16 {
        let w = mem.load_word(self.addr.wrapping_add(self.index));
        self.index = self.index.wrapping_add(2);
        w
    }

    pub fn fetch_byte_as_i8(&mut self, mem: &mut dyn MemoryIO) -> i8 {
        self.fetch_byte(mem) as i8
    }

    pub fn fetch_byte_as_i16(&mut self, mem: &mut dyn MemoryIO) -> i16 {
        i16::from(self.fetch_byte_as_i8(mem))
    }
}
