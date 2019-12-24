use super::mem::MemoryIO;

use super::isa_dbase;

#[derive(Debug,Clone)]
pub struct InstructionDecoder {
    index : u16,
    pub op_code : u16,
    pub cycles : u16,
    pub addr : u16,
    pub data: [u8; 5],
    pub next_addr : u16, 
    pub instruction_info :  &'static isa_dbase::Instruction,
}

// Decode an op
// Takes memory read as closure
// means we can destructively read op code when emulating
// or non destructively inspect for disassembly

fn decode_op(addr : u16, mut read : impl FnMut(u16)->u8) -> InstructionDecoder 
{
    // Array insrtuction bytes is copied to
    let mut data = [0,0,0,0,0];
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
        0x10 | 0x11 => { (a << 8) + fetch() }
        _ => a
    };

    // Fetch a reference to the extended infomation about this
    // opcode
    let instruction_info = isa_dbase::get(op_code);

    // Create the decoded instruction
    InstructionDecoder {
        next_addr : index,
        index,
        addr,
        op_code,
        instruction_info,
        cycles : instruction_info.cycles,
        data
    }
}

impl InstructionDecoder {

    pub fn new(_addr: u16)-> Self {
        panic!()
    }

    pub fn new_from_inspect_mem<M: MemoryIO>(addr: u16, mem : &M) -> Self {
        decode_op(addr, |addr| { mem.inspect_byte(addr) })
    }

    pub fn new_from_read_mem<M: MemoryIO>(addr: u16, mem : &mut M) -> Self {
        decode_op(addr, |addr| { mem.load_byte(addr) })
    }

    pub fn fetch_byte<M : MemoryIO>(&mut self, mem: &mut M) -> u8 {
        let b = mem.load_byte( self.addr.wrapping_add(self.index) );
        self.index = self.index.wrapping_add(1);
        b
    }

    pub fn fetch_word<M : MemoryIO>(&mut self, mem: &mut M) -> u16 {
        let w = mem.load_word( self.addr.wrapping_add(self.index) );
        self.index = self.index.wrapping_add(2);
        w   
    }

    pub fn fetch_byte_as_i8<M : MemoryIO>(&mut self, mem: &mut M) -> i8 {
        self.fetch_byte(mem) as i8
    }

    pub fn fetch_byte_as_i16<M : MemoryIO>(&mut self, mem: &mut M) -> i16 {
        i16::from(self.fetch_byte_as_i8(mem))
    }
}
