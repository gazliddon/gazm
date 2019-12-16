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

use crate::mem::*;

const IO_BASE   : u16   = 0x9800;
const IO_RASTER : u16   = 0x9830;
const IO_SW_1   : u16   = 0x9831;
const IO_SW_2   : u16   = 0x9832;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Color {
    r : u8,
    g : u8,
    b : u8,
}

#[derive(Clone, Copy)]
pub struct Io {
    pub palette : [u8 ; 16 * 3],
    halt : bool,
}

impl Io {
    pub fn new() -> Self {
        Self {
            palette : [0; 16 * 3] ,
            halt : false
        }
    }

    pub fn clear_halt(&mut self) {
        self.halt = false;
    }

    pub fn get_halt(&self) -> bool {
        self.halt
    }

    fn is_palette(addr : u16) -> bool {
        ( addr >= IO_BASE )  & ( addr < IO_BASE.wrapping_add(3 * 16) )
    }
}

impl MemoryIO for Io {

    fn inspect_byte(&self, addr:u16) -> u8 {
        if Io::is_palette(addr) {
            self.palette[addr.wrapping_sub(IO_BASE) as usize]
        } else {
            0
        }
    }

    fn upload(&mut self, _addr : u16, _data : &[u8]) {
        panic!("TBD")
    }

    fn get_range(&self) -> (u16, u16) {
        (IO_BASE, IO_BASE.wrapping_add(0xff))
    }

    fn update_sha1(&self, _digest : &mut Sha1) {
        panic!("TBD")
    }


    fn load_byte(&mut self, addr:u16) -> u8 {

        if Io::is_palette(addr) {
            self.palette[addr.wrapping_sub(IO_BASE) as usize]
        } else if addr == IO_RASTER {
            0xff
        } else {
            0
        }
    }

    fn store_byte(&mut self, addr:u16, val:u8) {
        if Io::is_palette(addr) {
            self.palette[addr.wrapping_sub(IO_BASE) as usize] = val
        } else if addr == IO_RASTER {
            // if you write to IO_RASTER the cpu will halt until vsync
            self.halt = true
        }
    }

    fn get_name(&self) -> String {
        "Io".to_string()
    }
}
