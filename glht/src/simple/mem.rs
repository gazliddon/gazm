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

use super::{emu, io};
// use super::{filewatcher, io, state, utils};
use emu::mem::memcore::MemoryIO;
use io::*;


////////////////////////////////////////////////////////////////////////////////
// Extend breakpoint to be initialisable from gdb bp descriptions

////////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum MemRegion {
    Illegal,
    Ram,
    IO,
    Screen,
}

pub struct SimpleMem {
    pub ram: emu::mem::MemBlock,
    pub screen: emu::mem::MemBlock,
    pub io: Io,
    addr_to_region: [MemRegion; 0x1_0000],
    name: String,
}


#[allow(dead_code)]
impl SimpleMem {
    pub fn new() -> Self {
        let screen = emu::mem::MemBlock::new("screen", false, 0x0000, 0x9800);
        let ram = emu::mem::MemBlock::new("ram", false, 0x9900, 0x1_0000 - 0x9900);
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

    pub fn get_region(&self, _addr: u16) -> &dyn MemoryIO {
        let region = self.addr_to_region[_addr as usize];

        use self::MemRegion::*;

        match region {
            Ram => &self.ram,
            IO => &self.io,
            Screen => &self.screen,
            Illegal => panic!("Illegal! inspect from {:02x}", _addr),
        }
    }

    pub fn get_region_mut(&mut self, _addr: u16) -> &mut dyn MemoryIO {
        let region = self.addr_to_region[_addr as usize];
        use self::MemRegion::*;

        match region {
            Ram => &mut self.ram,
            IO => &mut self.io,
            Screen => &mut self.screen,
            Illegal => panic!("Illegal! inspect from {:02x}", _addr),
        }
    }
}

impl MemoryIO for SimpleMem {
    fn upload(&mut self, addr: u16, data: &[u8]) {
        info!("Uploading data to addr {:04x}", addr);
        info!("top addr would be {:04x}", addr as usize + data.len() - 1);
        info!(
            "last two bytes would be {:02x} {:02x}",
            data[data.len() - 2],
            data[data.len() - 1]
        );

        let mut addr = addr;

        for i in data {
            self.store_byte(addr, *i);
            addr = addr.wrapping_add(1);
        }
    }

    fn get_range(&self) -> (u16, u16) {
        (0, 0xffff)
    }

    fn update_sha1(&self, _digest: &mut emu::sha1::Sha1) {
        unimplemented!("TBD")
    }

    fn inspect_byte(&self, addr: u16) -> u8 {
        let reg = self.get_region(addr);
        reg.inspect_byte(addr)
    }

    fn load_byte(&mut self, addr: u16) -> u8 {
        let reg = self.get_region_mut(addr);
        reg.load_byte(addr)
    }

    fn store_byte(&mut self, addr: u16, val: u8) {
        let reg = self.get_region_mut(addr);
        reg.store_byte(addr, val)
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}
