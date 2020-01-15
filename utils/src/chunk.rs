use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub file : String,
    pub line_number : usize,
}

impl Location {
    pub fn new(file : &str, line_number : usize) -> Self {
        Self {
            file : file.to_string(),
            line_number
        }
    }

    pub fn set_line_number(&mut self, line_number : usize) {
        self.line_number = line_number;
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub location : Location,
    pub addr : u16,
    pub data : Vec<u8>,
    pub last_addr : u16,
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "0x{:04x}", self.addr)?;
        writeln!(f, "\t{:?}", self.location)?;
        write!(f,"\t{:02X?}", self.data)
    }
}

impl Chunk {
    pub fn addr_range(&self) -> std::ops::Range<usize> {
        let addr = self.addr as usize;
        (addr..(addr+self.data.len()))
    }

    fn set_last_addr(&mut self) {
        let last_addr = self.addr as usize + self.data.len() - 1 ;
        if last_addr > 0xffff {
            panic!("Memory region exceeds 16 bits:\n{}", self)
        }
        self.last_addr = last_addr as u16;
    }

    pub fn new(addr : u16, data : Vec<u8>, file : &str, line_number : usize) -> Self {
        let location = Location::new(file, line_number);

        let mut ret = Self {
            addr, data, location,
            last_addr : 0
        };

        ret.set_last_addr();
        ret
    }

    pub fn collides(&self, other : &Self) -> bool {
        self.last_addr > other.addr && self.addr > other.last_addr
    }

    pub fn add_bytes(&mut self, bytes : Vec<u8> ) {
        self.data.extend(bytes);
        self.set_last_addr();
    }
}


