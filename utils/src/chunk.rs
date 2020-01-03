use std::fmt;

pub struct Chunk {
    addr : u16,
    data : Vec<u8>,
    source_file : String,
    line : usize,
    last_addr : u16,
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "0x{:04x}", self.addr)?;
        writeln!(f, "\t{}", self.source_file)?;
        writeln!(f, "\t{}", self.line)?;
        write!(f,"\t{:02X?}", self.data)
    }
}

impl Chunk {
    pub fn new(addr : u16, data : Vec<u8>, source_file : &str, line : usize) -> Self {
        let source_file = source_file.to_string();
        let last_addr = addr as usize + data.len() - 1 ;

        let ret = Self {
            addr, data,
            source_file,
            line,
            last_addr : last_addr as u16,
        };

        if last_addr > 0xffff {
            panic!("Memory region exceeds 16 bits:\n{}", ret)
        }

        ret
    }

    pub fn collides(&self, other : &Self) -> bool {
        self.last_addr > other.addr && self.addr > other.last_addr
    }

    pub fn add_bytes(&mut self, bytes : Vec<u8> ) {
        self.data.extend(bytes);
    }
}


