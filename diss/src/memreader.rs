use emu::mem::*;

pub struct MemReader<'a, T: MemoryIO> {
    mem: &'a mut T,
    addr: usize,
}

impl<'a, T: MemoryIO> MemReader<'a, T> {

    pub fn get_mem(&mut self) -> &mut T {
        self.mem
    }
    pub fn get_addr(&self) -> usize {
        self.addr
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr
    }

    pub fn new(mem: &'a mut T) -> Self {
        let addr = mem.get_range().start;
        Self { mem, addr }
    }

    pub fn peek_byte(&self) -> Result<u8, MemErrorTypes> {
        self.mem.inspect_byte(self.addr )
    }

    pub fn peek_word(&self) -> Result<u16, MemErrorTypes> {
        self.mem.inspect_word(self.addr )
    }

    pub fn next_byte(&mut self) -> Result<u8, MemErrorTypes> {
        let ret = self.mem.load_byte(self.addr );
        self.addr += 1;
        ret
    }
    pub fn next_word(&mut self) -> Result<u16, MemErrorTypes> {
        let ret = self.mem.load_word(self.addr );
        self.addr += 2;
        ret
    }
}
