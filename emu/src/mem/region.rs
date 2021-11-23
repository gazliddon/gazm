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
    pub fn checked_new(addr: u16, size: usize) -> Result<Self, RegionErr> {
        if size == 0 {
            return Err(RegionErr::SizeIsZero);
        }

        let (addr, last_addr) = calc_addr_last(addr, size);

        if last_addr > 0xffff {
            return Err(RegionErr::RegionToLargeToFit(addr, last_addr));
        }

        Ok(Self::new(addr as u16, size))
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }


    pub fn len(&self) -> usize {
        let (addr, last_addr) = self.to_usize();
        (last_addr - addr) + 1
    }

    pub fn new(addr: u16, mut size: usize) -> Self {
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

    pub fn as_range(&self) -> std::ops::RangeInclusive<usize> {
        let (addr, last_addr) = self.to_usize();
        addr..=last_addr
    }
}

fn calc_addr_last(addr: u16, size: usize) -> (usize, usize) {
    (addr as usize, addr as usize + size as usize - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_regions() {
        {
            let start : u16 = 0;
            let size = 0x10;

            let mr = Region::checked_new(start, size);
            assert!(mr.is_ok());
            let mr = mr.unwrap();
            assert!(mr.len() == size);

            let desired = start as usize..=( size - 1);
            let range = mr.as_range();

            assert!(range == desired, "wanted {:?} got {:?}", desired, range);

            assert!(mr.is_in_region(start));

            assert!(mr.is_in_region(( start as usize + size - 1 ) as u16));

            assert!(!mr.is_in_region(( start as usize + size ) as u16));
        }
    }

    #[test]
    fn size_zero() {
        let mr = Region::checked_new(0, 0);
        assert_eq!(mr, Err(RegionErr::SizeIsZero));
    }
#[test]
    fn contains() {
        let mr = Region::checked_new(0, 0x1_0000);
        assert!(mr.is_ok());
        let mr = mr.unwrap();
    }


    #[test]
    fn len() {
        let len: usize = 0x10;
        let addr: u16 = 0;
        let mr = Region::new(addr, len);
        assert_eq!(mr.len(), len as usize);

        let desired = addr as usize ..= (addr as usize +len -1 ) as usize;
            assert!(
                mr.as_range()
                    == desired
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
