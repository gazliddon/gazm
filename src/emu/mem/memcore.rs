// memory trait
use std::vec::Vec;
use std::ops::Range;
pub use sha1::Sha1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemError {
    AddressError(u16),
    IllegalWrite(u16),
    BreakPointRead(u16),
    BreakPointWrite(u16),
}

pub fn build_addr_to_region<E : Copy>(illegal : E, mem_tab :  &[(E, &dyn MemoryIO )]) -> [E; 0x1_0000] {

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

fn to_mem_range( address : u16, size :u16 ) -> Range<u32> {
    use std::cmp::min;
    let last_mem = u32::from(address) + u32::from(size);
    ( u32::from(address) .. min(0x1_0000, last_mem) )
}

pub fn as_word(lo : u8, hi : u8) -> u16 {
    u16::from(lo) | ( u16::from(hi) << 8 )
}

pub fn as_bytes(val : u16) -> (u8,u8) {
    ( (val&0xff) as u8, (val>>8) as u8 )
}


pub trait CheckedMemoryIo {
    fn inspect_byte(&self, _addr:u16) -> Result<(), MemError>;
    fn load_byte(&mut self, _addr:u16) -> Result<(), MemError>;
    fn store_byte(&mut self, _addr:u16, _val:u8) -> Result<(), MemError>;
}

pub trait MemoryIO {
    fn inspect_word(&self, _addr:u16) -> u16 {
        let lo = self.inspect_byte(_addr.wrapping_add(1));
        let hi = self.inspect_byte(_addr);
        as_word(lo, hi)
    }

    // Min implementation
    fn inspect_byte(&self, _addr:u16) -> u8 {
        panic!("TBD")
    }

    fn upload(&mut self, _addr : u16, _data : &[u8]);

    fn get_range(&self) -> (u16, u16);

    fn update_sha1(&self, _digest : &mut Sha1);

    fn load_byte(&mut self, _addr:u16) -> u8;
        
    fn store_byte(&mut self, _addr:u16, _val:u8);

    // Min implementation end


    fn get_name(&self) -> String {
        "default".to_string()
    }

    fn get_sha1_string(&self) -> String {
        let mut m = Sha1::new();
        self.update_sha1(&mut m);
        m.digest().to_string()
    }


    fn is_in_range(&self, _val : u16) -> bool {
        let (base, last) = self.get_range();
        (_val >= base) && (_val <= last)
    }


    fn store_word(&mut self, addr:u16, val:u16) {
        let (lo,hi) = as_bytes(val);
        self.store_byte(addr, hi);
        self.store_byte(addr.wrapping_add(1), lo);
    }

    fn load_word(&mut self, addr:u16) -> u16 {
        let lo = self.load_byte(addr.wrapping_add(1));
        let hi = self.load_byte(addr);
        as_word(lo, hi)
    }

    fn get_mem_as_str(&self, addr:u16, size:u16 ) -> String {
        let r = to_mem_range( addr, size);

        let mut v : Vec<String> = Vec::new();

        for a in r {
            let b = self.inspect_byte(a as u16);
            let t = format!("{:02X}", b);
            v.push(t);
        }

        v.join(" ")
    }
}

