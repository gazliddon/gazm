#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub addr: u16,
    pub last_addr: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegionErr {
    SizeIsZero,
    RegionToLargeToFit(usize, usize),
}

impl Region {
    pub fn checked_new(addr: u16, size: u16) -> Result<Self, RegionErr> {
        if size == 0 {
            return Err(RegionErr::SizeIsZero);
        }

        let (addr, last_addr) = calc_addr_last(addr, size);

        if last_addr > 0xffff {
            return Err(RegionErr::RegionToLargeToFit(addr, last_addr));
        }

        Ok(Self::new(addr as u16, size))
    }

    pub fn len(&self) -> usize {
        let (addr, last_addr) = self.to_usize();
        (last_addr - addr) + 1
    }

    pub fn new(addr: u16, mut size: u16) -> Self {
        if size == 0 {
            size = 1;
        }

        let (addr, mut last_addr) = calc_addr_last(addr, size);

        if last_addr >= 0x1_0000 {
            last_addr = 0xffff;
        }

        Self {
            addr: addr as u16,
            last_addr: last_addr as u16,
        }
    }

    pub fn is_in_region(&self, addr: u16) -> bool {
        addr >= self.addr && addr <= self.last_addr
    }

    fn to_usize(&self) -> (usize, usize) {
        (self.addr as usize, self.last_addr as usize)
    }

    pub fn as_range(&self) -> std::ops::Range<usize> {
        let (addr, last_addr) = self.to_usize();
        addr..last_addr
    }
}

fn calc_addr_last(addr: u16, size: u16) -> (usize, usize) {
    (addr as usize, addr as usize + size as usize - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_regions() {
        {
            let mr = Region::checked_new(0, 0x10);
            assert!(mr.is_ok());
            let mr = mr.unwrap();
            assert!(mr.len() == 0x10);
            assert!(
                mr.as_range()
                    == std::ops::Range {
                        start: 0,
                        end: 0x0f
                    }
            );
            assert!(mr.is_in_region(0));
            assert!(mr.is_in_region(0xf));
            assert!(!mr.is_in_region(0x10));
        }
    }

    #[test]
    fn size_zero() {
        let mr = Region::checked_new(0, 0);
        assert_eq!(mr, Err(RegionErr::SizeIsZero));
    }

    #[test]
    fn len() {
        let len: u16 = 0x10;
        let addr: u16 = 0;
        let mr = Region::new(addr, len);
        assert_eq!(mr.len(), len as usize);
            assert!(
                mr.as_range()
                    == std::ops::Range {
                        start: addr as usize,
                        end: ( addr + len - 1 ) as usize
                    }
            );
    }

    #[test]
    fn too_big() {
        let mr = Region::checked_new(0xffff, 0x2);

        assert!(
            mr == Err(RegionErr::RegionToLargeToFit(0xffff, 0x1_0000)),
            "value was {:04X?}",
            mr
        )
    }
}
