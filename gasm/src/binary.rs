

#[derive(Debug, Clone, PartialEq)]
pub enum AccessType {
    ReadWrite,
    ReadOnly,
}

#[derive(Debug, Clone)]
pub struct Binary {
    write_address: usize,
    written: bool,
    range: Option<(usize, usize)>,
    pub data: Vec<u8>,
    ref_data : Option<Vec<u8>>,
    write_offset: isize,
    watches : Vec<Watch>,
    access_type: AccessType,
}

impl Default for Binary {
    fn default() -> Self {
        Self::new(0x10000, AccessType::ReadWrite)
    }
}

use crate::as6809::{ MapFile, Record };

use emu::mem::LoggingMemMap;
use romloader::ResultExt;
use thiserror::Error;

pub struct MemoryLocation {
    physical: usize, logical: usize
}

impl std::fmt::Display for MemoryLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "physical: ${:05X} logical: ${:04X}", self.physical, self.logical)
    }
}

impl std::fmt::Debug for MemoryLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Error, Debug)]
pub enum BinaryError {
    #[error("{addr:04X?} (expected {expected:02X?}, found {val:02X?})")]
    DoesNotMatchReference {addr: usize, val : u8, expected: u8},
    #[error("Tried to write to {0:X?}")]
    InvalidWriteAddress(usize),
    #[error("Value {val} does not fit into a {dest_type}")]
    DoesNotFit {dest_type: String, val : i64},
    #[error("Hit watch location: {0}")]
    Halt(MemoryLocation),
    #[error("Write to read only memory: {0}")]
    IllegalWrite(MemoryLocation)
}

#[derive(Debug, Clone)]
pub struct Watch {
    range : std::ops::Range<usize>,
}

impl Binary {
    pub fn add_watch(&mut self, range : std::ops::Range<usize>) {
        self.watches.push(Watch{range})
    }

    pub fn bin_reference(&mut self, dest : usize, m : &[u8]) {
        if self.ref_data.is_none() {
            self.ref_data = Some(vec![0; self.data.len()]);
        }

        let bin = self.ref_data.as_mut().unwrap();

        for (i,x) in m.iter().enumerate() {
            bin[i+dest] = *x
        }
    }

    pub fn addr_reference(&mut self, m : crate::as6809::MapFile) {
        let  mut bin = vec![0; self.data.len()];

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

    pub fn new(size : usize, access_type: AccessType) -> Self {
        Self {
            write_address: 0,
            written: false,
            range: None,
            data: vec![0; size],
            ref_data: None,
            write_offset: 0,
            watches: vec![],
            access_type,
        }
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

    pub fn get_write_location(&self) -> MemoryLocation {
        MemoryLocation {
            logical : self.get_write_address(),
            physical : self.get_write_address_with_offset()
        }
    }

    pub fn write_byte(&mut self, val: u8) -> Result<(), BinaryError> {
        let loc = self.get_write_location();

        let physical = loc.physical;

        if self.access_type == AccessType::ReadOnly {
            return Err(BinaryError::IllegalWrite(loc))
        }

        if let Some((mut low, mut high)) = self.range {
            if physical < low {
                low = physical
            }

            if physical > high {
                high = physical
            }

            self.range = Some((low, high))
        } else {
            self.range = Some((physical, physical))
        }

        if physical >= self.data.len() {
            panic!("Address out of bounds!")
        }

        for r in &self.watches {
            if r.range.contains(&physical) {
                let x = BinaryError::Halt(loc);
                return Err(x)
            }
        }

        self.data[physical] = val;
        self.write_address += 1;

        if let Some(ref_data) = &self.ref_data {
            if ref_data[physical] != self.data[physical] {
                return Err(
                    BinaryError::DoesNotMatchReference{addr:physical, expected: ref_data[physical], val : self.data[physical]}
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

