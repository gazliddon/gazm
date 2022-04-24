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

use super::io;
use byteorder::ByteOrder;
use emu::mem::{MemErrorTypes, MemResult, MemoryIO};
use imgui_glium_renderer::imgui::sys::{igIsRectVisibleVec2, igIsWindowAppearing};
use io::*;
use super::region::*;

////////////////////////////////////////////////////////////////////////////////
// Extend breakpoint to be initialisable from gdb bp descriptions

////////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////////

/// Find the Region this write would go to depending on the mapping
/// register
fn get_region_write(addr: usize, _mapping: &Mapping) -> Region {
    let r = Region::new(addr);
    // Can't write to RomHi or palette
    match r {
        Region::RomHi => Region::Illegal,
        Region::Counter => Region::Illegal,

        _ => r,
    }
}

/// Find the Region reading addr would come from taking into account the mapping
/// register
fn get_region_read(addr: usize, mapping: &Mapping) -> Region {
    let r = Region::new(addr);

    match r {
        // Can't read from watchdog
        Region::Watchdog => Region::Illegal,

        // Region read comes from dependings on the mapping reg
        Region::RomLo => match mapping {
            Mapping::RomRead => Region::RomLo,
            Mapping::RamRead => Region::RamLo,
        },
        _ => r,
    }
}

/// Creates a table of memory address -> (read region, write region)
/// for the range r with the Mapping of m
fn make_region_tab(r: std::ops::Range<usize>, m: &Mapping) -> Vec<(Region, Region)> {
    let mut ret: Vec<_> = vec![];

    for addr in r {
        let read = get_region_read(addr, m);
        let write = get_region_write(addr, m);
        ret.push((read, write))
    }

    ret
}

/// How 0 - 0x9800 reads are handled
pub enum Mapping {
    RomRead,
    RamRead,
}

pub struct SimpleMem<E: ByteOrder> {
    pub io: Io,
    name: String,
    mapping: Mapping,
    addr_to_region_rom: Vec<(Region, Region)>,
    addr_to_region_ram: Vec<(Region, Region)>,

    ram_lo: emu::mem::MemBlock<E>,
    rom_lo: emu::mem::MemBlock<E>,
    rom_hi: emu::mem::MemBlock<E>,
    ram_hi: emu::mem::MemBlock<E>,
    palette: emu::mem::MemBlock<E>,
}

impl<E: ByteOrder> Default for SimpleMem<E> {
    fn default() -> Self {
        use emu::mem::MemBlock;
        use log::info;

        let name = "simple".to_string();
        let io = Io::new();

        let rom_lo = MemBlock::new("rom_lo", true, &ROM_LO_SCREEN);
        let ram_lo = MemBlock::new("ram_lo", false, &ROM_LO_SCREEN);
        let rom_hi = MemBlock::new("rom_hi", true, &ROM_HI);
        let ram_hi = MemBlock::new("ram_hi", false, &RAM_HI);
        let palette = MemBlock::new("palette", false, &PALETTE);

        SimpleMem {
            name,
            addr_to_region_rom: make_region_tab(0..0x1_0000, &Mapping::RomRead),
            addr_to_region_ram: make_region_tab(0..0x1_0000, &Mapping::RamRead),
            io,
            mapping: Mapping::RomRead,
            rom_lo,
            ram_lo,
            rom_hi,
            ram_hi,
            palette,
        }
    }
}

#[allow(dead_code)]
impl<E: ByteOrder> SimpleMem<E> {
    fn upload_rom_byte(&mut self, addr: usize, data: u8) -> MemResult<()> {
        if ROM_LO_SCREEN.contains(&addr) {
            self.rom_lo.store_byte(addr, data)
        } else if ROM_HI.contains(&addr) {
            self.rom_hi.store_byte(addr, data)
        } else {
            MemResult::Err(MemErrorTypes::IllegalWrite(addr))
        }
    }

    pub fn upload_rom(&mut self, addr: usize, data: &[u8]) -> MemResult<()> {
        for (i, b) in data.iter().enumerate() {
            let a = addr + i;
            let res = self.upload_rom_byte(a, *b);

            if !res.is_ok() {
                println!("Cannot write {b} to 0x{a:04X}")
            }
            res?;
        }
        Ok(())
    }

    pub fn get_screen(&self) -> &emu::mem::MemBlock<E> {
        &self.ram_lo
    }

    fn get_region_enum(&self, addr: usize) -> Option<(Region, Region)> {
        let r = match self.mapping {
            Mapping::RomRead => self.addr_to_region_rom.get(addr),
            Mapping::RamRead => self.addr_to_region_ram.get(addr),
        };
        r.cloned()
    }

    fn get_region_mut(&mut self, r: &Region) -> &mut dyn MemoryIO {
        match r {
            Region::RomLo => &mut self.rom_lo,
            Region::RamLo => &mut self.ram_lo,
            Region::RomHi => &mut self.rom_hi,
            Region::RamHi => &mut self.ram_hi,
            Region::Palette => &mut self.palette,
            _ => panic!("Fucked"),
        }
    }

    fn get_region(&self, r: &Region) -> &dyn MemoryIO {
        match r {
            Region::RomLo => &self.rom_lo,
            Region::RamLo => &self.ram_lo,
            Region::RomHi => &self.rom_hi,
            Region::RamHi => &self.ram_hi,
            _ => panic!("Fucked"),
        }
    }

    fn get_region_read_mut(&mut self, addr: usize) -> &mut dyn MemoryIO {
        let (r, _) = self.get_region_enum(addr).unwrap();
        self.get_region_mut(&r)
    }

    fn get_region_write_mut(&mut self, addr: usize) -> &mut dyn MemoryIO {
        let (_, r) = self.get_region_enum(addr).unwrap();
        self.get_region_mut(&r)
    }
    fn get_region_read(&self, addr: usize) -> &dyn MemoryIO {
        let (r, _) = self.get_region_enum(addr).unwrap();
        self.get_region(&r)
    }

    fn get_region_write(&self, addr: usize) -> &dyn MemoryIO {
        let (_, r) = self.get_region_enum(addr).unwrap();
        self.get_region(&r)
    }
}

impl<E: ByteOrder> MemoryIO for SimpleMem<E> {
    fn inspect_word(&self, addr: usize) -> MemResult<u16> {
        let reg = self.get_region_read(addr);
        reg.inspect_word(addr)
    }

    fn upload(&mut self, _addr: usize, _data: &[u8]) -> MemResult<()> {
        panic!()
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        0..0x1_0000
    }

    fn update_sha1(&self, _digest: &mut emu::sha1::Sha1) {
        unimplemented!("TBD")
    }

    fn inspect_byte(&self, addr: usize) -> MemResult<u8> {
        let reg = self.get_region_read(addr);
        reg.inspect_byte(addr)
    }

    fn load_byte(&mut self, addr: usize) -> MemResult<u8> {
        let reg = self.get_region_read_mut(addr);
        reg.load_byte(addr)
    }

    fn store_byte(&mut self, addr: usize, val: u8) -> MemResult<()> {
        let reg = self.get_region_write_mut(addr);
        reg.store_byte(addr, val)
    }

    fn load_word(&mut self, addr: usize) -> MemResult<u16> {
        let reg = self.get_region_read_mut(addr);
        reg.load_word(addr)
    }

    fn store_word(&mut self, addr: usize, val: u16) -> MemResult<()> {
        let reg = self.get_region_write_mut(addr);
        reg.store_word(addr, val)
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}
