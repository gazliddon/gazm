use super::{MemErrorTypes, MemMap, MemMapIO, MemoryIO, Region};
use sha1::Sha1;
use std::vec::Vec;

pub struct MemBlock {
    pub read_only: bool,
    pub data: Vec<u8>,
    pub name: String,
    pub region : Region,
}

#[allow(dead_code)]
impl MemBlock {
    pub fn new(name: &str, read_only: bool, base: u16, size: u16) -> MemBlock {
        let data = vec![0u8;size as usize];
        Self::from_data(base, name, &data , read_only)
    }

    pub fn from_data(addr: u16, name: &str, data: &[u8], read_only: bool) -> MemBlock {
        let size = data.len();

        let mr = Region::checked_new(addr, size ).unwrap();

        MemBlock {
            read_only,
            data : data.to_vec(),
            name: name.to_string(),
            region : mr
        }
    }
    fn to_index(&self, addr : u16) -> usize {
        assert!(self.region.is_in_region(addr));
        addr as usize - self.region.addr as usize
    }
}

#[allow(dead_code)]
impl MemMap {
    pub fn add_mem_block(&mut self, name: &str, writable: bool, base: u16, size: u16) {
        let mb = Box::new(MemBlock::new(name, writable, base, size));
        self.add_memory(mb);
    }
}

#[allow(dead_code)]
impl MemoryIO for MemBlock {
    fn inspect_byte(&self, addr: u16) -> Result<u8, MemErrorTypes> {
        let i = self.to_index(addr);
        let d = self.data[i];
        Ok(d)
    }

    fn update_sha1(&self, digest: &mut Sha1) {
        digest.update(&self.data);
    }

    fn upload(&mut self, _addr: u16, _data: &[u8]) -> Result<(), MemErrorTypes> {
        panic!("not done")
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_range(&self) -> std::ops::RangeInclusive<usize> {
        self.region.as_range()
    }

    fn load_byte(&mut self, addr: u16) -> Result<u8, MemErrorTypes> {
        let i = self.to_index(addr);
        let v = self.data[i];
        Ok(v)
    }

    fn store_byte(&mut self, addr: u16, val: u8) -> Result<(), MemErrorTypes> {
        let idx = self.to_index(addr);
        self.data[idx] = val;
        Ok(())
    }
}
