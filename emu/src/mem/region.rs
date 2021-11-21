
pub struct Region {
    pub addr : u16,
    pub last_addr : u16,
}

pub enum RegionErr {
    SizeIsZero,
    RegionToLargeToFit(usize, usize),
}

impl Region {
    pub fn checked_new(addr : u16, size : u16) -> Result<Self, RegionErr> {
        if size == 0 {
            return Err(RegionErr::SizeIsZero)
        }

        {
            let addr = addr as usize;
            let last_addr = addr + size as usize;
            if last_addr > 0xffff {
                return Err(RegionErr::RegionToLargeToFit(addr,last_addr))
            }
        }

        Ok( Self::new(addr, size) )
    }

    pub fn len(&self) -> usize {
        ( self.last_addr - self.addr ) as usize
    }

    pub fn new( addr : u16, size : u16 ) -> Self {
        let size = if size == 0 {
            1 as usize
        } else {
            size as usize
        };

        let addr = addr as usize;
        let mut last_addr = addr + size as usize;

        if last_addr >= 0x1_0000 {
            last_addr = 0xffff;
        }

        Self {
            addr : addr as u16,
            last_addr : last_addr as u16
        }
    }
    fn to_usize(&self) -> (usize, usize) {
        (self.addr as usize, self.last_addr as usize)
    }

    pub fn as_range(&self) -> std::ops::Range<usize> {
        let (addr, last_addr) = self.to_usize();
        addr..last_addr
    }
}

