// use mem::Memory;
use super::{ MemoryIO, MemErrorTypes };
use sha1::Sha1;
use std::fmt;

pub trait MemMapIO {
    fn add_memory(&mut self, mem: Box<dyn MemoryIO>);
}

#[derive(Default)]
pub struct MemMap {
    all_memory: Vec<Box<dyn MemoryIO>>,
    name: String,
}

impl fmt::Debug for MemMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut strs: Vec<String> = Vec::new();

        for m in &self.all_memory {
            strs.push(m.get_name().clone())
        }

        write!(f, "{}", strs.join(" "))
    }
}

impl MemoryIO for MemMap {
    fn update_sha1(&self, digest: &mut Sha1) {
        for m in &self.all_memory {
            m.update_sha1(digest);
        }
    }

    fn upload(&mut self, addr: u16, data: &[u8]) -> Result<(), MemErrorTypes>{
        for (i, item) in data.iter().enumerate() {
            let a = addr.wrapping_add(i as u16);
            self.store_byte(a, *item)?;
        }
        Ok(())
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        0..0xffff
    }

    fn load_byte(&mut self, addr: u16) -> Result<u8, MemErrorTypes> {
        for m in &mut self.all_memory {
            if m.is_in_range(addr) {
                return m.load_byte(addr);
            }
        }
        Err(MemErrorTypes::IllegalAddress(addr))
    }

    fn store_byte(&mut self, addr: u16, val: u8) -> Result<(), MemErrorTypes>{
        for m in &mut self.all_memory {
            if m.is_in_range(addr) {
                return m.store_byte(addr, val)
            }
        }
        Err(MemErrorTypes::IllegalAddress(addr))
    }
}

#[allow(dead_code)]
impl MemMap {
    pub fn new() -> MemMap {
        MemMap {
            all_memory: Vec::new(),
            name: "all memory".to_string(),
        }
    }

    // pub fn load_roms<'a>(&mut self, roms : &[(&'a str, u16)]) -> &mut Self{
    //     use utils::load_file;
    //     for rom in roms.iter() {
    //         let data = load_file(rom.0);
    //         self.upload(rom.1, &data);
    //     }
    //     self
    // }
}

impl MemMapIO for MemMap {
    fn add_memory(&mut self, mem: Box<dyn MemoryIO>) {
        self.all_memory.push(mem)
    }
}
