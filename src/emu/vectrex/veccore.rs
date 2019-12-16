// #![allow(single_match)]

use clap::{ArgMatches};
use std::cell::RefCell;
use std::rc::Rc;

use crate::gdbstub;
use crate::diss::Disassembler;
use crate::mem::*;
use crate::cpu::{Regs, StandardClock, Clock, InstructionDecoder};
use crate::cpu;

use crate::m6522::M6522;
use crate::vectrex::window;
use crate::vectrex::dac;

use crate::vectrex::dac::Dac;



#[derive(Debug, Clone, Copy, PartialEq)]
enum MemRegion {
    Illegal,
    Ram,
    Rom,
    Cart,
    VIA,
}

static FAST_ROM: &[u8] = include_bytes!("../../resources/fastrom.dat");
static SYS_ROM: &[u8]  = include_bytes!("../../resources/rom.dat");

// Contains memory and memmapped perihpherals
// decodes memory map

struct VecMem<C : Clock> {
    via            : M6522<C>,
    dac            : dac::Dac,
    sys_rom        : MemBlock,
    cart_rom       : MemBlock,
    ram            : MemBlock,
    addr_to_region : [MemRegion; 0x1_0000],
    name           : String,
}



impl<C: Clock> VecMem<C> {

    pub fn new(rc_clock : &Rc<RefCell<C>>) -> VecMem<C> {

        info!("creaying vecmem");

        let name = "VecMem".to_string();

        let via = M6522::new(0xd000,0x800, rc_clock);

        let sys_rom   = MemBlock::from_data(0xe000, "sys_rom", FAST_ROM, false);
        let cart_rom  = MemBlock::new("cart", true, 0, 16 * 1024);
        let ram       = MemBlock::new("ram", false, 0xc800, 1024);

        let dac       = Dac {};

        let addr_to_region = {

            use self::MemRegion::*;

            let mems : &[(MemRegion, &dyn MemoryIO )] = &[
                (Rom, &sys_rom ),
                (Cart, &cart_rom ), 
                (Ram, &ram ),
                (VIA, &via) ];

            build_addr_to_region(Illegal, mems)
        };

        info!("created vecmem");

        VecMem {
            sys_rom, cart_rom, ram, dac, name,via, addr_to_region
        }
    }

    fn get_region(&self, _addr : u16) -> &dyn MemoryIO {
        let region = self.addr_to_region[_addr as usize];

        use self::MemRegion::*;

        match region {
            Ram     => &self.ram,
            Rom     => &self.sys_rom,
            Cart    => &self.cart_rom,
            VIA     => &self.via,
            Illegal => panic!("Illegal! read from {:02x}", _addr),
        }
    }

    fn get_region_mut(&mut self, _addr : u16) -> &mut dyn MemoryIO {
        let region = self.addr_to_region[_addr as usize];

        use self::MemRegion::*;

        match region {
            Ram     => &mut self.ram,
            Rom     => &mut self.sys_rom,
            Cart    => &mut self.cart_rom,
            VIA     => &mut self.via,
            Illegal => panic!("Illegal! read from {:02x}", _addr),
        }
    }
}

impl<C : Clock> MemoryIO for VecMem<C> {

    fn upload(&mut self, _addr : u16, _data : &[u8]) {
        unimplemented!("TBD")
    }

    fn get_range(&self) -> (u16, u16) {
        (0, 0xffff)
    }

    fn update_sha1(&self, _digest : &mut Sha1) {
        unimplemented!("TBD")
    }

    fn inspect_byte(&self, addr:u16) -> u8 {
        let region = self.get_region(addr);
        region.inspect_byte(addr)
    }

    fn load_byte(&mut self, addr:u16) -> u8 {
        let region = self.get_region_mut(addr);
        region.load_byte(addr)
    }

    fn store_byte(&mut self, addr:u16, val:u8) {
        let region = self.get_region_mut(addr);
        region.store_byte(addr, val)
    }

    fn store_word(&mut self, addr:u16, val:u16) {
        let ah = addr.wrapping_add(1);

        let r1 = self.addr_to_region[addr as usize];
        let r2 = self.addr_to_region[ah as usize];

        if r1 == r2 {
            let region = self.get_region_mut(addr);
            region.store_word(addr, val)

        } else {
            self.store_byte(addr, ( val >> 8 ) as u8);
            self.store_byte(ah, val as u8);
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

pub struct Vectrex {
    regs        : Regs,
    rc_clock    : Rc<RefCell<StandardClock>>,
    vec_mem     : VecMem<StandardClock>,
    window      : window::Window,
    gdb_enabled : bool,
}

fn mk_data_mem(addr : u16 ,name : &str, data : &[u8], writeable : bool ) -> Box<dyn MemoryIO> {
    Box::new(MemBlock::from_data(addr, name, data, writeable))
}

impl gdbstub::DebuggerHost for Vectrex {
    fn do_break(&mut self) {
    }

    fn force_pc(&mut self, _pc : u16)  {
    }

    fn resume(&mut self)  -> gdbstub::Sigs {
        panic!("kjsakjska")
    }

    fn set_step(&mut self)  -> gdbstub::Sigs {
        panic!("kslkslaks")
    }

    fn add_breakpoint(&mut self, _addr : u16)  {

    }
    fn add_write_watchpoint (&mut self, _addr : u16) {

    }
    fn add_read_watchpoint(&mut self, _addr : u16) {

    }
    fn del_breakpoint(&mut self, _addr : u16)  {
    }

    fn del_write_watchpoint(&mut self, _addr : u16)  {
    }

    fn del_read_watchpoint(&mut self, _addr : u16)  {
    }

    fn examine(&self, addr : u16) -> u8  {
        self.vec_mem.inspect_byte(addr)
    }

    fn write (&mut self, addr : u16, val : u8) {
        self.vec_mem.store_byte(addr, val)
    }

    fn read_registers(&self, reply : &mut gdbstub::Reply) {

        let regs = &self.regs;

        let cc = regs.flags.bits();

        reply.push_u8(cc);
        reply.push_u8(regs.a);
        reply.push_u8(regs.b);
        reply.push_u8(regs.dp);

        reply.push_u16(regs.x);
        reply.push_u16(regs.y);
        reply.push_u16(regs.u);
        reply.push_u16(regs.s);
        reply.push_u16(regs.pc);
    }

    fn write_registers(&mut self, _data : &[u8]) {
    }

    fn get_reg(&self, _reg_num : usize) -> u16 {
        unimplemented!("get_reg r = {}", _reg_num)
    }

    fn set_reg(&self, _r_num : usize, _val : u16) {
        unimplemented!("set_reg r = {}  v = {}", _r_num, _val)
    }
}

impl Vectrex {

    pub fn new() -> Vectrex {

        let window = window::Window::new();

        let rc_clock = Rc::new(RefCell::new(StandardClock::new(1_500_000)));

        let vec_mem = VecMem::new(&rc_clock);
        info!("back from vecmen!");

        let mut ret = Vectrex {
            rc_clock, vec_mem, window,
            regs  : Regs::new(),
            gdb_enabled : false
        };

        ret.reset();

        ret
    }

    pub fn from_matches(matches : &ArgMatches) -> Vectrex {
        // use std::thread;
        // use std::net::TcpListener;
        // use gdbstub::GdbRemote;
        // use std::sync::mpsc;

        let mut ret = Vectrex::new();

        info!("back from vecmen");

        let gdb_enabled = matches.is_present("enable-gdb");

        info!("gdb {}", gdb_enabled);

        ret.gdb_enabled = gdb_enabled;

        info!("done reset");

        ret
    }

    pub fn run(&mut self) {

    }

    pub fn update(&mut self) -> InstructionDecoder {
        let ins = cpu::step(&mut self.regs, &mut self.vec_mem, &self.rc_clock);

        if let Ok(ins) = ins {
            ins
        } else {
            panic!("fucked")
        }
    }

    pub fn step(&mut self) -> InstructionDecoder {

        let mut diss = Disassembler::new();

        let pc = self.regs.pc;
        let (_, txt) =  diss.diss(&mut self.vec_mem, pc, None);

        if let Ok(ins) = cpu::step(&mut self.regs, &mut self.vec_mem, &self.rc_clock) {
        if self.vec_mem.via.is_dirty() {
            println!("${:04x}   {:20} : {} ",  pc, txt, self.regs);
            println!();
            self.vec_mem.via.clear_dirty();
        }
        ins

        } else {
            panic!("fix this!");

        }
    }

    pub fn reset(&mut self) {

        cpu::reset(&mut self.regs, &mut self.vec_mem);
    }
}

