use crate::as6809::{MapFile, Record};
use emu::mem::LoggingMemMap;
use thiserror::Error;
use serde_derive::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    vec,
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum AccessType {
    ReadWrite,
    ReadOnly,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinRef {
    pub file: PathBuf,
    pub start: usize,
    pub size: usize,
    pub dest: usize,
}

#[derive(Debug, Clone)]
struct BinRefChunk {
    physical_range: std::ops::Range<usize>,
    ref_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Binary {
    write_address: usize,
    written: bool,
    range: Option<(usize, usize)>,
    pub data: Vec<u8>,
    write_offset: isize,
    watches: Vec<Watch>,
    access_type: AccessType,
    bin_refs: Vec<BinRefChunk>,
    unchecked_writes: Vec<MemoryLocation>,
}

impl Default for Binary {
    fn default() -> Self {
        Self::new(0x10000, AccessType::ReadWrite)
    }
}


#[derive(Clone)]
pub struct MemoryLocation {
    pub physical: usize,
    pub logical: usize,
}

impl std::fmt::Display for MemoryLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "physical: ${:05X} logical: ${:04X}",
            self.physical, self.logical
        )
    }
}

impl std::fmt::Debug for MemoryLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Error, Debug)]
pub enum BinaryError {
    #[error("${addr:04X?} (${logical_addr:05X})(expected ${expected:02X?}, found ${val:02X?})")]
    DoesNotMatchReference {
        addr: usize,
        logical_addr: usize,
        val: u8,
        expected: u8,
    },
    #[error("Tried to write to {0:X?}")]
    InvalidWriteAddress(usize),
    #[error("Value {val} does not fit into a {dest_type}")]
    DoesNotFit { dest_type: String, val: i64 },
    #[error("Hit watch location: {0}")]
    Halt(MemoryLocation),
    #[error("Write to read only memory: {0}")]
    IllegalWrite(MemoryLocation),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WriteStatus {
    Checked,
    NoCheck,
}

#[derive(Debug, Clone)]
pub struct Watch {
    range: std::ops::Range<usize>,
}

impl Binary {
    pub fn add_watch(&mut self, range: std::ops::Range<usize>) {
        self.watches.push(Watch { range })
    }

    pub fn bin_reference(&mut self, bin_ref: &BinRef, m: &[u8]) {
        let chunk = BinRefChunk {
            physical_range: bin_ref.dest..bin_ref.dest + bin_ref.size,
            ref_data: m.into(),
        };

        self.bin_refs.push(chunk);
    }

    pub fn addr_reference(&mut self, m: crate::as6809::MapFile) {
        let mut bin = vec![0; self.data.len()];

        for Record { addr, data } in m.data {
            for (i, d) in data.iter().enumerate() {
                bin[addr as usize + i] = *d;
            }
        }
    }

    fn dirty(&mut self) {
        self.written = true;
    }

    pub fn new(size: usize, access_type: AccessType) -> Self {
        Self {
            write_address: 0,
            written: false,
            range: None,
            data: vec![0; size],
            write_offset: 0,
            watches: vec![],
            access_type,
            bin_refs: vec![],
            unchecked_writes: vec![],
        }
    }

    pub fn bump_write_address(&mut self, n: usize) {
        self.write_address += n;
    }

    pub fn get_write_address(&self) -> usize {
        self.write_address
    }

    pub fn get_write_address_with_offset(&self) -> usize {
        self.logical_to_physical(self.write_address)
    }

    pub fn get_write_offset(&self) -> isize {
        self.write_offset
    }

    pub fn set_write_address(&mut self, pc: usize, offset: isize) {
        self.write_address = pc;
        self.set_write_offset(offset)
    }
    pub fn skip(&mut self, skip: usize) {
        self.write_address += skip;
    }

    pub fn get_range(&self) -> Option<(usize, usize)> {
        self.range
    }

    pub fn set_write_offset(&mut self, offset: isize) {
        self.write_offset = offset
    }

    pub fn logical_to_physical(&self, addr: usize) -> usize {
        (addr as isize + self.write_offset) as usize
    }

    fn write_byte_check(
        &mut self,
        val: i64,
        r: std::ops::Range<i64>,
        dest_type: &str,
    ) -> Result<WriteStatus, BinaryError> {
        if r.contains(&val) {
            self.write_byte(val as u8)
        } else {
            Err(BinaryError::DoesNotFit {
                dest_type: dest_type.to_string(),
                val,
            })
        }
    }

    pub fn write_ibyte_check_size(&mut self, val: i64) -> Result<WriteStatus, BinaryError> {
        let x = 1 << 7;
        let r = -x..x;
        self.write_byte_check(val, r, "i8")
    }
    pub fn write_byte_check_size(&mut self, val: i64) -> Result<WriteStatus, BinaryError> {
        let bits = 8;
        let end = 1 << bits;
        let start = -(1 << (bits - 1));
        self.write_byte_check(val, start..end, "i8 or u8")
    }

    pub fn write_ubyte_check_size(&mut self, val: i64) -> Result<WriteStatus, BinaryError> {
        let x = 1 << 8;
        let r = 0..x;
        self.write_byte_check(val, r, "u8")
    }

    fn write_word_check(
        &mut self,
        val: i64,
        r: std::ops::Range<i64>,
        dest_type: &str,
    ) -> Result<WriteStatus, BinaryError> {
        if r.contains(&val) {
            self.write_word(val as u16)
        } else {
            Err(BinaryError::DoesNotFit {
                dest_type: dest_type.to_string(),
                val,
            })
        }
    }

    pub fn get_unchecked_writes(&self) -> Vec<MemoryLocation> {
        let mut x = self.unchecked_writes.clone();
        x.sort_by(|a,b| 
            a.logical.cmp(&b.logical)
            );
        x
    }

    pub fn write_iword_check_size(&mut self, val: i64) -> Result<WriteStatus, BinaryError> {
        let x = 1 << 15;
        let r = -x..x;
        self.write_word_check(val, r, "i16")
    }
    pub fn write_word_check_size(&mut self, val: i64) -> Result<WriteStatus, BinaryError> {
        let bits = 16;
        let end = 1 << bits;
        let start = -(1 << (bits - 1));
        self.write_word_check(val, start..end, "i16 or u16")
    }
    pub fn write_uword_check_size(&mut self, val: i64) -> Result<WriteStatus, BinaryError> {
        let x = 1 << 16;
        let r = 0..x;
        self.write_word_check(val, r, "u16")
    }

    pub fn get_write_location(&self) -> MemoryLocation {
        MemoryLocation {
            logical: self.get_write_address(),
            physical: self.get_write_address_with_offset(),
        }
    }

    pub fn write_byte(&mut self, val: u8) -> Result<WriteStatus, BinaryError> {
        let loc = self.get_write_location();

        let physical = loc.physical;
        let logical = loc.logical;

        if self.access_type == AccessType::ReadOnly {
            return Err(BinaryError::IllegalWrite(loc));
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
                return Err(x);
            }
        }

        self.data[physical] = val;
        self.write_address += 1;

        let mut did_check = WriteStatus::NoCheck;

        for x in &self.bin_refs {
            if x.physical_range.contains(&physical) {
                did_check = WriteStatus::Checked;
                let expected = x.ref_data[physical - x.physical_range.start];
                if expected != val {
                    return Err(BinaryError::DoesNotMatchReference {
                        addr: physical,
                        logical_addr: logical,
                        expected,
                        val,
                    });
                }
            }
        }

        if did_check == WriteStatus::NoCheck {
            self.unchecked_writes.push(MemoryLocation {
                physical,
                logical 
            })
        }

        Ok(did_check)
    }

    pub fn fill(&mut self, count: usize, byte: u8) -> Result<(), BinaryError> {
        for _i in 0..count {
            self.write_byte(byte)?;
        }
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<WriteStatus, BinaryError> {
        let mut did_check = WriteStatus::Checked;
        for b in bytes {
            let ch = self.write_byte(*b)?;
            if ch == WriteStatus::NoCheck {
                did_check = ch
            }
        }
        Ok(did_check)
    }

    // Get bytes from memory
    // address is the physical_address
    pub fn get_bytes(&self, physical_address: usize, count: usize) -> &[u8] {
        let r = physical_address..(physical_address + count);
        &self.data[r]
    }

    pub fn write_word(&mut self, val: u16) -> Result<WriteStatus, BinaryError> {
        let hi = val >> 8;
        let lo = val & 0xff;
        self.write_byte(hi as u8)?;
        self.write_byte(lo as u8)
    }
}
