use super::{Regs, Flags};

use std::fmt;

impl Regs {
    pub fn get_hdr() -> String {
        "PC   D    A  B  X    Y    U    S    DP : flags".to_string()
    }
}

impl fmt::Display for Regs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\n{:04x} {:04x} {:02x} {:02x} {:04x} {:04x} {:04x} {:04x} {:02x} : {}",
            Self::get_hdr(),
            self.pc,
            self.get_d(), 
            self.a, 
            self.b, 
            self.x,
            self.y,
            self.u,
            self.s,
            self.dp,
            self.flags)
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08b} : {:?}", self.bits(), *self)
    }
}
