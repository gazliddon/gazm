// Memory map
pub const ROM_LO_SCREEN: std::ops::Range<usize> = 0..0x9000;
pub const RAM_LO: std::ops::Range<usize> = 0x9000..0xc000;
pub const PALETTE: std::ops::Range<usize> = 0xc000..0xc010;
pub const PIA0: std::ops::Range<usize> = 0xc804..0xc808;
pub const PIA1: std::ops::Range<usize> = 0xc80c..0xc810;
pub const NVRAM: std::ops::Range<usize> = 0xc900..0xca00;
pub const COUNTER: std::ops::Range<usize> = 0xcb00..0xcc00;
pub const WATCHDOG: std::ops::Range<usize> = 0xcbff..0xcc00;
pub const RAM_HI: std::ops::Range<usize> = 0xcc00..0xd000;
pub const ROM_HI: std::ops::Range<usize> = 0xd000..0x1_0000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Region {
    RomLo,
    RamLo,
    Palette,
    Pia0,
    Pia1,
    NVRAM,
    Counter,
    Watchdog,
    RomHi,
    RamHi,
    Illegal,
}

impl Region {
    pub fn new(addr : usize) -> Self {
        addr.into()
    }
}

impl Default for Region {
    fn default() -> Self {
        Region::Illegal
    }
}

impl From<usize> for Region {
    fn from(addr: usize) -> Self {
        if ROM_LO_SCREEN.contains(&addr) {
            return Region::RomLo;
        }
        if RAM_LO.contains(&addr) {
            return Region::RamLo;
        }

        if PALETTE.contains(&addr) {
            return Region::Palette;
        }

        if PIA0.contains(&addr) {
            return Region::Pia0;
        }

        if PIA1.contains(&addr) {
            return Region::Pia1;
        }

        if NVRAM.contains(&addr) {
            return Region::NVRAM;
        }

        if COUNTER.contains(&addr) {
            return Region::Counter;
        }

        if WATCHDOG.contains(&addr) {
            return Region::Watchdog;
        }

        if ROM_HI.contains(&addr) {
            return Region::RomHi;
        }

        if RAM_HI.contains(&addr) {
            return Region::RamHi;
        }

        Region::Illegal
        
    }

}
