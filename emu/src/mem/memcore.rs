// memory trait
pub use sha1::Sha1;
use std::ops::Range;
use std::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemErrorTypes {
    IllegalAddress(u16),
    IllegalWrite(u16),
    IllegalRead(u16),
}

pub type MemResult<T> = std::result::Result<T,MemErrorTypes>;

pub fn build_addr_to_region<X: Copy>(illegal: X, mem_tab: &[(X, &dyn MemoryIO)]) -> [X; 0x1_0000] {
    let mut ret = [illegal; 0x1_0000];

    for (i, id) in ret.iter_mut().enumerate() {
        for &(this_id, mem) in mem_tab {
            if mem.is_in_range(i as u16) {
                *id = this_id;
            }
        }
    }
    ret
}

fn to_mem_range(address: u16, size: u16) -> Range<u32> {
    use std::cmp::min;
    let last_mem = u32::from(address) + u32::from(size);
    u32::from(address)..min(0x1_0000, last_mem)
}


pub trait CheckedMemoryIo {
    fn inspect_byte(&self, _addr: u16) -> MemResult<u16>;
    fn load_byte(&mut self, _addr: u16) -> MemResult<u8>;
    fn store_byte(&mut self, _addr: u16, _val: u8) -> MemResult<()>;
}

pub trait MemoryIO {
    fn inspect_word(&self, _addr: u16) -> MemResult<u16> {
        todo!()
    }

    // Min implementation
    fn inspect_byte(&self, _addr: u16) -> MemResult<u8> {
        panic!("TBD")
    }

    fn upload(&mut self, _addr: u16, _data: &[u8]) -> MemResult<()>;

    fn get_range(&self) -> std::ops::RangeInclusive<usize>;

    fn update_sha1(&self, _digest: &mut Sha1);

    fn load_byte(&mut self, _addr: u16) -> MemResult<u8>;

    fn store_byte(&mut self, _addr: u16, _val: u8) -> MemResult<()>;

    // Min implementation end

    fn get_name(&self) -> String {
        "default".to_string()
    }

    fn is_valid_addr(&self, addr : u16) -> bool {
        self.is_in_range(addr)
    }

    fn get_sha1_string(&self) -> String {
        let mut m = Sha1::new();
        self.update_sha1(&mut m);
        m.digest().to_string()
    }

    fn is_in_range(&self, addr : u16) -> bool {
        self.get_range().contains(&( addr as usize ))
    }

    fn store_word(&mut self, addr: u16, val: u16) -> MemResult<()>;

    fn load_word(&mut self, addr: u16) -> MemResult<u16>;

    fn get_mem_as_str(&self, addr: u16, size: u16) -> String {
        let r = to_mem_range(addr, size);

        let mut v: Vec<String> = Vec::new();

        for a in r {
            let b = self.inspect_byte(a as u16).unwrap();
            let t = format!("{:02X}", b);
            v.push(t);
        }

        v.join(" ")
    }
}
