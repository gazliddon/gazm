use super::mem::MemoryIO;

#[derive(Debug,Clone,Default)]
pub struct InstructionDecoder {
    pub op_code : u16,
    pub cycles : u32,
    pub addr : u16,
    pub bytes : usize,
    pub next_addr : u16,
}

impl InstructionDecoder {

    pub fn new(addr: u16)-> Self {
        InstructionDecoder {
            addr,
            next_addr : addr,
            cycles : 2,
            .. Default::default()
        }
    }
    fn bump_fetch(&mut self, v : usize) {
        self.next_addr = self.next_addr.wrapping_add(v as u16);
        self.bytes +=  1;
    }

    pub fn add_cycles(&mut self, i : u32)  {
        let r = self.cycles.wrapping_add(i);
        self.cycles = r;
    }

    pub fn inc_cycles(&mut self) {
        self.add_cycles(1);
    }

    pub fn fetch_byte<M : MemoryIO>(&mut self, mem: &mut M) -> u8 {
        let b = mem.load_byte(self.next_addr);
        self.bump_fetch(1);
        b
    }

    pub fn fetch_byte_as_i8<M : MemoryIO>(&mut self, mem: &mut M) -> i8 {
        self.fetch_byte(mem) as i8
    }

    pub fn fetch_byte_as_i16<M : MemoryIO>(&mut self, mem: &mut M) -> i16 {
        i16::from(self.fetch_byte_as_i8(mem))
    }


    pub fn fetch_word<M : MemoryIO>(&mut self, mem: &mut M) -> u16 {
        let w = mem.load_word(self.next_addr);
        self.bump_fetch(2);
        w   
    }

    pub fn fetch_instruction<M: MemoryIO>(&mut self, mem: &mut M) -> u16 {
        self.cycles = 2;

        let a = u16::from(self.fetch_byte(mem));

        self.op_code = match a {
            0x10 | 0x11 => {
                self.inc_cycles();
                (a << 8) + u16::from(self.fetch_byte(mem))
            }
            _ => a
        };

        self.op_code
    }
}
