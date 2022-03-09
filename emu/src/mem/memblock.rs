use std::marker::PhantomData;

use super::{MemResult, MemoryIO, Region};
use byteorder::ByteOrder;
use sha1::Sha1;

pub struct MemBlock<E: ByteOrder> {
    pub read_only: bool,
    pub data: Vec<u8>,
    pub name: String,
    pub region: Region,
    phanton: PhantomData<E>,
}

#[allow(dead_code)]
impl<E: ByteOrder> MemBlock<E> {
    pub fn new(name: &str, read_only: bool, base: u16, size: u32) -> MemBlock<E> {
        let data = vec![0u8; size as usize];
        Self::from_data(base, name, &data, read_only)
    }

    pub fn from_data(addr: u16, name: &str, data: &[u8], read_only: bool) -> MemBlock<E> {
        let size = data.len();

        let mr = Region::checked_new(addr, size).unwrap();

        MemBlock {
            read_only,
            data: data.to_vec(),
            name: name.to_string(),
            region: mr,
            phanton: Default::default(),
        }
    }
    fn to_index(&self, addr: u16) -> usize {
        assert!(self.region.is_in_region(addr));
        addr as usize - self.region.addr as usize
    }
}

#[allow(dead_code)]
impl<E: ByteOrder> MemoryIO for MemBlock<E> {
    fn inspect_byte(&self, addr: u16) -> MemResult<u8> {
        let i = self.to_index(addr);
        let d = self.data[i];
        Ok(d)
    }

    fn update_sha1(&self, digest: &mut Sha1) {
        digest.update(&self.data);
    }

    fn upload(&mut self, _addr: u16, _data: &[u8]) -> MemResult<()> {
        panic!("not done")
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        self.region.as_range()
    }

    fn load_byte(&mut self, addr: u16) -> MemResult<u8> {
        let i = self.to_index(addr);
        let v = self.data[i];
        Ok(v)
    }

    fn store_byte(&mut self, addr: u16, val: u8) -> MemResult<()> {
        let idx = self.to_index(addr);
        self.data[idx] = val;
        Ok(())
    }

    fn store_word(&mut self, addr: u16, val: u16) -> MemResult<()> {
        let mut buf = [0; 2];
        E::write_u16(&mut buf, val);
        self.store_byte(addr, buf[0])?;
        self.store_byte(addr.wrapping_add(1), buf[1])
    }

    fn load_word(&mut self, addr: u16) -> MemResult<u16> {
        let a = self.load_byte(addr)?;
        let b = self.load_byte(addr.wrapping_add(1))?;
        let buf = [a, b];
        Ok(E::read_u16(&buf))
    }
}
