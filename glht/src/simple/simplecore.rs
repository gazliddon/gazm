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

use byteorder::ByteOrder;
// use filewatcher::FileWatcher;
//
use emu::{cpu, diss, mem};

use emu::breakpoints::BreakPoints;
use mem::MemoryIO;

use log::info;

use super::mem::SimpleMem;
use super::{filewatcher, state};

use cpu::{CpuErr, Regs, StandardClock};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum MachineErr {
    Cpu(CpuErr),
    BreakPoint(u16),
    Halted,
}

impl From<CpuErr> for MachineErr {
    fn from(c: CpuErr) -> Self {
        MachineErr::Cpu(c)
    }
}

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

const W: usize = 304;
const H: usize = 256;
const SCR_BYTES: usize = W * H * 3;

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

fn check_breakpoints(machine: &(impl Machine + ?Sized)) -> Result<(), MachineErr> {
    let bp = machine.get_breakpoints();
    let pc = machine.get_regs().pc;

    if bp.has_any_breakpoint(pc) {
        Err(MachineErr::BreakPoint(pc))
    } else {
        Ok(())
    }
}

pub trait Machine {
    fn get_breakpoints(&self) -> &BreakPoints;
    fn get_breakpoints_mut(&mut self) -> &mut BreakPoints;

    fn get_state(&self) -> SimState;
    fn set_state(&mut self, state: SimState) -> Option<SimState>;

    fn get_rom(&self) -> &romloader::Rom;
    fn get_mem(&self) -> &dyn MemoryIO;
    fn get_mem_mut(&mut self) -> &mut dyn MemoryIO;
    fn get_clock_mut(&mut self) -> &mut Rc<RefCell<StandardClock>>;
    fn get_context_mut(&mut self) -> cpu::Context;
    fn update(&mut self) -> Result<(), MachineErr>;

    fn get_regs(&self) -> &cpu::Regs;

    fn get_dissambler(&self) -> diss::Disassembler {
        panic!("")
    }

    fn run_instructions(&mut self, n: usize) -> Result<(), MachineErr> {
        for _ in 0..n {
            let mut ctx = self.get_context_mut();
            ctx.step()?;
            let pc = ctx.get_pc();
            let bp = self.get_breakpoints();
            if bp.has_any_breakpoint(pc) {
                return Err(MachineErr::BreakPoint(pc));
            }
        }

        Ok(())
    }

    fn step(&mut self) -> Result<u16, MachineErr> {
        self.get_context_mut().step()?;
        Ok(self.get_regs().pc)
    }

    fn reset(&mut self) {
        let mut ctx = self.get_context_mut();
        ctx.reset();
    }
}

#[allow(dead_code)]
pub struct SimpleMachine<M: MemoryIO> {
    regs: Regs,
    mem: M,
    rc_clock: Rc<RefCell<StandardClock>>,
    watcher: Option<filewatcher::FileWatcher>,
    events: Vec<SimEvent>,
    dirty: bool,
    verbose: bool,
    state: state::State<SimState>,
    rom: romloader::Rom,
    breakpoints: emu::breakpoints::BreakPoints,
}

impl< M: MemoryIO> Machine for SimpleMachine<M> {
    fn get_breakpoints(&self) -> &BreakPoints {
        &self.breakpoints
    }

    fn get_breakpoints_mut(&mut self) -> &mut BreakPoints {
        &mut self.breakpoints
    }

    fn get_regs(&self) -> &cpu::Regs {
        &self.regs
    }

    fn get_rom(&self) -> &romloader::Rom {
        &self.rom
    }
    fn get_mem(&self) -> &dyn MemoryIO{
        &self.mem
    }

    fn get_mem_mut(&mut self) -> &mut dyn MemoryIO {
        &mut self.mem
    }

    fn get_clock_mut(&mut self) -> &mut Rc<RefCell<StandardClock>> {
        &mut self.rc_clock
    }

    fn get_context_mut(&mut self) -> cpu::Context {
        emu::cpu::Context::new(&mut self.mem, &mut self.regs)
    }

    fn update(&mut self) -> Result<(), MachineErr> {
        use self::SimEvent::*;

        while let Some(event) = self.events.pop() {
            if self.state.get() == SimState::Quitting {
                break;
            }

            match event {
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
                if let Err(err) = self.run_instructions(1000) {
                    self.set_state(SimState::Paused);
                    return Err(err);
                }

                // self.update_texture();
            }

            SimState::Paused => {}
        };

        Ok(())
    }

    fn get_state(&self) -> SimState {
        self.state.get()
    }

    fn set_state(&mut self, state: SimState) -> Option<SimState> {
        self.state.set(state);
        self.state.get_previous()
    }
}

#[allow(dead_code)]
impl< M: MemoryIO> SimpleMachine<M> {
    fn add_event(&mut self, event: SimEvent) {
        self.events.push(event)
    }

    fn toggle_verbose(&mut self) {
        let v = self.verbose;
        self.verbose = !v;
    }

    pub fn new(mem: M, rom: romloader::Rom) -> Self {

        let path = std::env::current_dir().expect("getting dir");
        info!("Creatning Simple 6809 machine");
        info!("cd = {}", path.display());

        let clock = cpu::StandardClock::new(2_000_000);
        let rc_clock = Rc::new(RefCell::new(clock));
        let regs = Regs::new();
        let verbose = false;

        Self {
            rom,
            mem,
            regs,
            rc_clock,

            verbose,
            watcher: None,
            events: vec![],
            dirty: false,
            state: state::State::new(SimState::Paused),
            breakpoints: emu::breakpoints::BreakPoints::new(),
        }
    }
}

fn load_rom_to_mem<M: MemoryIO>(file: &str, mem: M, addr: u16, size: usize) -> SimpleMachine<M> {
    let mut mem = mem;
    let rom = romloader::Rom::from_sym_file(file).expect("Load syms");
    info!("loaded symbol file {} as ROM", file);
    let rom_data = rom.get_slice(addr, size);
    mem.upload(addr, rom_data).unwrap();
    let mut ret = SimpleMachine::new(mem, rom);
    ret.reset();
    ret
}

pub fn make_simple<E: ByteOrder>(file: &str) -> SimpleMachine<SimpleMem<E>> {
    let mem = SimpleMem::default();
    load_rom_to_mem(file, mem, 0x9900, 0x1_0000 - 0x9900)
}
