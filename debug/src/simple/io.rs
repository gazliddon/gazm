/*
IO
    9800 -> 982F = Palette ram - 16 * RGB byte per col = 0x30]

    9830   R get raster hpos, W halt until vsync? (maybe raster pos?)

    9831  switches 1
                b0 = Up
                b1 = Down
                b2 = Left
                b3 = Right
                b4 = Fire 1
                b5 = Fire 2
    9831  switches 2
*/

use emu;

use emu::mem::{ MemoryIO, MemResult };
use emu::sha1::Sha1;

#[allow(dead_code)]
const IO_BASE: usize = 0x9800;
#[allow(dead_code)]
const IO_RASTER: usize = 0x9830;
#[allow(dead_code)]
const IO_SW_1: usize = 0x9831;
#[allow(dead_code)]
const IO_SW_2: usize = 0x9832;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Io {
    pub palette: [u8; 16 * 3],
    halt: bool,
}

impl Default for Io {
    fn default() -> Self {
        Self {
            palette: [0; 16 * 3],
            halt: false,
        }
    }

}

#[allow(dead_code)]
impl Io {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_halt(&mut self) {
        self.halt = false;
    }

    pub fn get_halt(&self) -> bool {
        self.halt
    }

    fn is_palette(addr: usize) -> bool {
        let r = IO_BASE .. IO_BASE+3*16;
        r.contains(&addr)
    }
}

use byteorder::ByteOrder;

impl MemoryIO for Io {
    fn inspect_byte(&self, addr: usize) -> MemResult<u8> {
        let r = if Io::is_palette(addr) {
            self.palette[addr.wrapping_sub(IO_BASE) as usize]
        } else {
            0
        };
        Ok(r)
    }

    fn inspect_word(&self, _addr: usize) -> MemResult<u16> {
        panic!()
    }

    fn upload(&mut self, _addr: usize, _data: &[u8]) -> MemResult<()>{
        panic!("TBD")
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        IO_BASE as usize .. IO_BASE.wrapping_add(0x100) as usize
    }

    fn update_sha1(&self, _digest: &mut Sha1) {
        panic!("TBD")
    }

    fn load_byte(&mut self, addr: usize) -> MemResult<u8> {
        let r = if Io::is_palette(addr) {
            self.palette[addr.wrapping_sub(IO_BASE) as usize]
        } else if addr == IO_RASTER {
            0xff
        } else {
            0
        };
        Ok(r)
    }
    
    fn load_word(&mut self, _addr: usize) -> MemResult<u16> {
        todo!()
    }

    fn store_byte(&mut self, addr: usize, val: u8) -> MemResult<()>{
        if Io::is_palette(addr) {
            self.palette[addr.wrapping_sub(IO_BASE) as usize] = val
        } else if addr == IO_RASTER {
            // if you write to IO_RASTER the cpu will halt until vsync
            self.halt = true
        }
        Ok(())
    }

    fn store_word(&mut self, _addr: usize, _val: u16) -> MemResult<()>{ 
        todo!()
    }

    fn get_name(&self) -> String {
        "Io".to_string()
    }
}