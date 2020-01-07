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

use super::{cpu, filewatcher, io, mem, state, utils, diss};
use clap::ArgMatches;
use mem::memcore::MemoryIO;

use io::*;

use cpu::{Regs, StandardClock};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum SimState {
    Paused,
    Running,
    Quitting,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SimEvent {
    HitSync,
    Halt,
    Pause,
    Quit,
    RomChanged,
    MaxCycles,
    Reset,
    ToggleVerbose,
    Run,
}

////////////////////////////////////////////////////////////////////////////////
// Extend breakpoint to be initialisable from gdb bp descriptions

////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
const W: usize = 304;
#[allow(dead_code)]
const H: usize = 256;
#[allow(dead_code)]
const DIMS: (u32, u32) = (W as u32, H as u32);
#[allow(dead_code)]
const SCR_BYTES: usize = W * H * 3;

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
    pub ram: mem::MemBlock,
    pub screen: mem::MemBlock,
    pub io: Io,
    addr_to_region: [MemRegion; 0x1_0000],
    name: String,
}

#[allow(dead_code)]
fn pix_to_rgb(p: u8, palette: &[u8], dest: &mut [u8]) {
    let p = p as usize;
    let palette = &palette[p * 3..];
    dest.copy_from_slice(&palette[..3]);
}

#[allow(dead_code)]
fn to_rgb(mem: &[u8], palette: &[u8]) -> [u8; SCR_BYTES] {
    let mut ret: [u8; SCR_BYTES] = [0; SCR_BYTES];

    for (i, b) in mem.iter().enumerate() {
        let x = (i / H) * 2;
        let y = i % H;
        let d = (x + y * W) * 3;

        let dest = &mut ret[d..];

        pix_to_rgb(b & 0xf, palette, &mut dest[..3]);
        pix_to_rgb(b >> 4, palette, &mut dest[3..6]);
    }

    ret
}

#[allow(dead_code)]
impl SimpleMem {
    pub fn new() -> Self {
        let screen = mem::MemBlock::new("screen", false, 0x0000, 0x9800);
        let ram = mem::MemBlock::new("ram", false, 0x9900, 0x1_0000 - 0x9900);
        let name = "simple".to_string();
        let io = Io::new();

        let addr_to_region = {
            use self::MemRegion::*;

            let mems: &[(MemRegion, &dyn mem::MemoryIO)] =
                &[(IO, &io), (Screen, &screen), (Ram, &ram)];

            mem::build_addr_to_region(Illegal, mems)
        };

        SimpleMem {
            ram,
            screen,
            name,
            addr_to_region,
            io,
        }
    }

    fn get_region(&self, _addr: u16) -> &dyn mem::MemoryIO {
        let region = self.addr_to_region[_addr as usize];

        use self::MemRegion::*;

        match region {
            Ram => &self.ram,
            IO => &self.io,
            Screen => &self.screen,
            Illegal => panic!("Illegal! inspect from {:02x}", _addr),
        }
    }

    fn get_region_mut(&mut self, _addr: u16) -> &mut dyn mem::MemoryIO {
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

impl mem::MemoryIO for SimpleMem {
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

#[allow(dead_code)]
pub struct Simple {
    regs: Regs,
    mem: SimpleMem,
    rc_clock: Rc<RefCell<StandardClock>>,
    file: Option<String>,
    watcher: Option<filewatcher::FileWatcher>,
    events: Vec<SimEvent>,
    dirty: bool,
    verbose: bool,
    state: state::State<SimState>,
}

#[allow(dead_code)]
impl Simple {
    pub fn new() -> Self {
        let clock = cpu::StandardClock::new(2_000_000);

        let rc_clock = Rc::new(RefCell::new(clock));
        let mem = SimpleMem::new();
        let regs = Regs::new();

        let verbose = false;

        let path = std::env::current_dir().expect("getting dir");
        info!("Creatning Simple 6809 machine");
        info!("cd = {}", path.display());

        let mut ret = Simple {
            mem,
            regs,
            rc_clock,
            verbose,
            file: None,
            watcher: None,
            events: vec![],
            dirty: false,
            state: state::State::new(SimState::Paused),
        };

        let file = "./asm/out/all.bin";
        let addr: u16 = 0x9900;

        ret.upload(file, addr);
        info!(
            "0xfff0 region = {}",
            ret.mem.get_region(0x9900).get_mem_as_str(0xfff0, 16)
        );
        ret.reset();

        ret
    }

    fn upload(&mut self, file: &str, addr: u16) {
        let bytes = std::fs::read(file).expect("Can't load rom");
        self.mem.upload(addr, &bytes);
        info!("Uploaded {} to 0x{:04x}", file, addr);
    }

    pub fn get_context_mut(&mut self) -> cpu::Context<StandardClock, SimpleMem> {
        cpu::Context::new(&mut self.mem, &mut self.regs, &self.rc_clock)
    }

    pub fn get_dissambler(&self) -> diss::Disassembler<SimpleMem> {
        diss::Disassembler::new(&self.mem)
    }

    pub fn step(&mut self) -> Option<SimEvent> {
        let mut ctx = self.get_context_mut();
        ctx.step().expect("Can't step");
        Some(SimEvent::Halt)
    }

    pub fn reset(&mut self) {
        let mut ctx = self.get_context_mut();
        ctx.reset();
        info!("Reset\n\t{}", self.regs);
    }

    fn handle_file_watcher(&mut self) {
        let mut has_changed = false;

        if let Some(ref mut watcher) = self.watcher {
            if watcher.has_changed() {
                has_changed = true;
            }
        }

        if has_changed {
            self.add_event(SimEvent::RomChanged);
        }
    }

    fn rom_changed(&mut self) {
        self.load_rom();
        self.reset();
    }

    fn load_rom(&mut self) {
        if let Some(ref file) = self.file {
            let data = utils::load_file(&file);
            self.mem.upload(0x9900, &data);
            info!("Loaded ROM: {}", file);
        }
    }

    pub fn from_matches(matches: &ArgMatches) -> Self {
        let mut ret = Self::new();
        let file = matches.value_of("ROM FILE").unwrap();
        ret.file = Some(file.to_string());
        ret.load_rom();
        ret.reset();

        if matches.is_present("watch-rom") {
            info!("Adding watch for rom file");
            let watcher = filewatcher::FileWatcher::new(file);
            ret.watcher = Some(watcher);
        }

        ret
    }

    fn run_to_sync(&mut self, max_instructions: usize) -> Option<SimEvent> {
        let mut ctx = self.get_context_mut();

        for _ in 0..max_instructions {
            ctx.step().expect("Can't step");
        }
        None
    }

    fn add_event(&mut self, event: SimEvent) {
        self.events.push(event)
    }

    fn toggle_verbose(&mut self) {
        let v = self.verbose;
        self.verbose = !v;
    }

    pub fn update_texture(&mut self) {
        let _buffer = {
            let scr = &self.mem.screen.data;
            let pal = &self.mem.io.palette;
            to_rgb(scr, pal)
        };

        // self.win.update_texture(&buffer);
    }

    pub fn update(&mut self) -> SimState {
        use self::SimEvent::*;

        self.handle_file_watcher();

        while let Some(event) = self.events.pop() {
            if self.state.get() == SimState::Quitting {
                break;
            }

            match event {
                RomChanged => self.rom_changed(),
                ToggleVerbose => self.toggle_verbose(),
                Pause => self.state.set(SimState::Paused),
                Quit => self.state.set(SimState::Quitting),
                Halt => self.state.set(SimState::Paused),
                Run => self.state.set(SimState::Running),
                _ => (),
            };
        }

        match self.state.get() {
            SimState::Quitting => {}

            SimState::Running => {
                self.run_to_sync(2_000_000 / 60);
                self.update_texture();
            }

            SimState::Paused => {}
        };

        self.state.get()
    }
}



