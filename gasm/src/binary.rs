
pub struct Binary {
    write_address: usize,
    written: bool,
    range: Option<(usize, usize)>,
    pub data: Vec<u8>,
    ref_data : Option<Vec<u8>>,
    write_offset: isize,
}

impl Default for Binary {
    fn default() -> Self {
        Self {
            write_address: 0,
            written: false,
            range: None,
            data: vec![0; 0x10000],
            ref_data: None,
            write_offset: 0,
        }
    }
}

use crate::as6809::{ MapFile, Record };

use romloader::ResultExt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinaryError {
    #[error("{addr:04X?} (expected {expected:02X?}, found {val:02X?})")]
    DoesNotMatchReference {addr: usize, val : u8, expected: u8},
    #[error("Tried to write to {0:X?}")]
    InvalidWriteAddress(usize),
    #[error("Value {val} does not fit into a {dest_type}")]
    DoesNotFit {dest_type: String, val : i64},
    #[error("Hit watch location")]
    Halt,
}

impl Binary {
    pub fn bin_reference(&mut self, dest : usize, m : &[u8]) {
        if self.ref_data.is_none() {
            self.ref_data = Some(vec![0; 0x10000]);
        }

        let bin = self.ref_data.as_mut().unwrap();

        for (i,x) in m.iter().enumerate() {
            bin[i+dest] = *x
        }
    }

    pub fn addr_reference(&mut self, m : crate::as6809::MapFile) {
        let  mut bin = vec![0; 0x10000];

        for Record {addr, data} in m.data {
            for (i,d) in data.iter().enumerate() {
                bin[addr as usize + i] = *d;
            }
        }

        self.ref_data = Some(bin);
    }

    fn dirty(&mut self) {
        self.written = true;
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn bump_write_address(&mut self, n: usize) {
        self.write_address += n;
    }

    pub fn get_write_address(&self) -> usize {
        self.write_address
    }

    pub fn get_write_address_with_offset(&self) -> usize {
        ( self.write_address as isize + self.write_offset ) as usize
    }

    pub fn set_write_address(&mut self, pc: usize, offset : isize) {
        self.write_address = pc;
        self.set_write_offset(offset)
    }
    pub fn skip(&mut self, skip : usize) {
        self.write_address += skip;
    }

    pub fn get_range(&self) -> Option<(usize, usize)> {
        self.range
    }

    pub fn set_write_offset(&mut self, offset : isize) {
        self.write_offset = offset
    }

    fn write_byte_check(&mut self, val: i64, r: std::ops::Range<i64>, dest_type : &str) -> Result<(), BinaryError> {
        if r.contains(&val) {
            self.write_byte(val as u8)
        } else {
            Err(BinaryError::DoesNotFit{dest_type: dest_type.to_string(), val})
        }
    }

    pub fn write_ibyte_check_size(&mut self, val: i64) -> Result<(),BinaryError> {
        let x = 1 << 7;
        let r = -x..x;
        self.write_byte_check(val, r, "i8")
    }
    pub fn write_byte_check_size(&mut self, val: i64) -> Result<(), BinaryError> {
        let bits = 8;
        let end = 1 << bits;
        let start = -(1<<(bits-1));
        self.write_byte_check(val, start..end, "i8 or u8")
    }

    pub fn write_ubyte_check_size(&mut self, val: i64) -> Result<(), BinaryError> {
        let x = 1 << 8;
        let r = 0..x;
        self.write_byte_check(val, r, "u8")
    }


    fn write_word_check(&mut self, val: i64, r: std::ops::Range<i64>, dest_type : &str) -> Result<(), BinaryError> {
        if r.contains(&val) {
            self.write_word(val as u16)
        } else {
            Err(BinaryError::DoesNotFit{dest_type: dest_type.to_string(), val})
        }
    }

    pub fn write_iword_check_size(&mut self, val: i64) -> Result<(), BinaryError> {
        let x = 1 << 15;
        let r = -x..x;
        self.write_word_check(val, r, "i16")
    }
    pub fn write_word_check_size(&mut self, val: i64) -> Result<(), BinaryError> {
        let bits = 16;
        let end = 1 << bits;
        let start = -(1<<(bits-1));
        self.write_word_check(val, start..end, "i16 or u16")
    }
    pub fn write_uword_check_size(&mut self, val: i64) -> Result<(), BinaryError> {
        let x = 1 << 16;
        let r = 0..x;
        self.write_word_check(val, r, "u16")
    }

    pub fn write_byte(&mut self, val: u8) -> Result<(), BinaryError> {

        let addr = self.write_address;


        if let Some((mut low, mut high)) = self.range {
            if addr < low {
                low = addr
            }

            if addr > high {
                high = addr
            }

            self.range = Some((low, high))
        } else {
            self.range = Some((addr, addr))
        }

        let new_addr = self.get_write_address_with_offset();

        if new_addr >= self.data.len() {
            panic!("Address out of bounds!")
        }

        self.data[new_addr] = val;
        self.write_address += 1;


        if let Some(ref_data) = &self.ref_data {
            if ref_data[new_addr] != self.data[new_addr] {
                return Err(
                    BinaryError::DoesNotMatchReference{addr:new_addr, expected: ref_data[new_addr], val : self.data[new_addr]}
                )
            }
        }

        Ok(())
    }

    pub fn fill(&mut self, count: usize, byte: u8) -> Result<(), BinaryError> {
        for _i in 0..count {
            self.write_byte(byte)?;
        }
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), BinaryError>{
        for b in bytes {
            self.write_byte(*b)?
        }
        Ok(())
    }

    pub fn get_bytes(&self, pc: usize, count: usize) -> &[u8] {
        &self.data[pc..(pc + count)]
    }

    pub fn write_word(&mut self, val: u16) -> Result<(), BinaryError>{
        let hi = val >> 8;
        let lo = val &0xff;
        self.write_byte(hi as u8)?;
        self.write_byte(lo as u8)
    }
}

