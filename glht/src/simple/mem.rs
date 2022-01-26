/*
 * Simple 6809 machine to test code on

    0000 -> 97ff Screen (304 * 256 pixels / 4bbpp)
    9800 -> 98ff IO
    9900 -> FFFF RAM 6700 (26k)

IO
    9800 -> 982F = Palette ram - 16 * RGB byte per col = 0x30]
    9830  raster pos
    9831  switches 1
                b0 = Up
                b1 = Down
                b2 = Left
                b3 = Right
                b4 = Fire 1
                b5 = Fire 2
    9831  switches 2

*/
// use emu::cpu;

// use filewatcher::FileWatcher;
//

use super::io;
use byteorder::ByteOrder;
// use super::{filewatcher, io, state, utils};
use emu::mem::{ MemoryIO, MemErrorTypes, MemResult };
use io::*;


////////////////////////////////////////////////////////////////////////////////
// Extend breakpoint to be initialisable from gdb bp descriptions

////////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum MemRegion {
    Illegal,
    Ram,
    IO,
    Screen,
}

pub struct SimpleMem<E: ByteOrder> {
    ram: emu::mem::MemBlock<E>,
    screen: emu::mem::MemBlock<E>,
    pub io: Io,
    addr_to_region: [MemRegion; 0x1_0000],
    name: String,
}

impl<E: ByteOrder> Default for SimpleMem<E> {

    fn default() -> Self {
        use log::info;
        let ram = emu::mem::MemBlock::new("ram", false, 0x9900, ( 0x1_0000 - 0x9900 ) as u32);
        info!("ram is {:04X?}", ram.region);

        let screen = emu::mem::MemBlock::new("screen", false, 0x0000, 0x9800);
        let name = "simple".to_string();
        let io = Io::new();

        let addr_to_region = {
            use self::MemRegion::*;

            let mems: &[(MemRegion, &dyn MemoryIO)] =
                &[(IO, &io), (Screen, &screen), (Ram, &ram)];

            emu::mem::build_addr_to_region(Illegal, mems)
        };

        SimpleMem {
            ram,
            screen,
            name,
            addr_to_region,
            io,
        }
    }

}

#[allow(dead_code)]
impl<E: ByteOrder> SimpleMem<E> {

    pub fn get_screen(&self) -> &emu::mem::MemBlock<E> {
        &self.screen
    }

    fn get_region(&self, addr: u16) -> &dyn MemoryIO {
        let region = self.addr_to_region[addr as usize];

        use self::MemRegion::*;

        match region {
            Ram => &self.ram,
            IO => &self.io,
            Screen => &self.screen,
            Illegal => panic!("Illegal! inspect from {:02x}", addr),
        }
    }

    // TODO turn this to Result return

    fn get_region_mut(&mut self, addr: u16) -> &mut dyn MemoryIO {
        let region = self.addr_to_region[addr as usize];
        use self::MemRegion::*;

        match region {
            Ram => &mut self.ram,
            IO => &mut self.io,
            Screen => &mut self.screen,
            Illegal => panic!("Illegal! inspect from {:02x}", addr),
        }
    }
}

impl<E: ByteOrder> MemoryIO for SimpleMem<E> {
    fn upload(&mut self, addr: u16, data: &[u8]) -> MemResult<()>{
        let mut addr = addr;

        for i in data {
            self.store_byte(addr, *i)?;
            addr = addr.wrapping_add(1);
        }
        Ok(())
    }

    fn is_valid_addr(&self, addr : u16) -> bool {
        let region = self.addr_to_region[addr as usize];
        region != self::MemRegion::Illegal
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        0..0x1_0000
    }

    fn update_sha1(&self, _digest: &mut emu::sha1::Sha1) {
        unimplemented!("TBD")
    }

    fn inspect_byte(&self, addr: u16) -> MemResult<u8> {
        let reg = self.get_region(addr);
        reg.inspect_byte(addr)
    }

    fn load_byte(&mut self, addr: u16) -> MemResult<u8> {
        let reg = self.get_region_mut(addr);
        reg.load_byte(addr)
    }

    fn store_byte(&mut self, addr: u16, val: u8) -> MemResult<()> {
        let reg = self.get_region_mut(addr);
        reg.store_byte(addr, val)
    }
    
    fn load_word(&mut self, addr: u16) -> MemResult<u16> {
        let reg = self.get_region_mut(addr);
        reg.load_word(addr)
    }

    fn store_word(&mut self, addr: u16, val: u16) -> MemResult<()> {
        let reg = self.get_region_mut(addr);
        reg.store_word(addr, val)
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

