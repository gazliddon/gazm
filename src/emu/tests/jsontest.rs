use crate::mem::{MemoryIO, LoggingMemMap, LogEntry, MemMap};
use crate::cpu::{Regs, StandardClock};
use crate::diss::Disassembler;
use clap::{ArgMatches};

use crate::cpu::step;

use std::cell::RefCell;
use std::rc::Rc;

use crate::tests::tester;
use crate::proclog::{Step};
use separator::Separatable;

// use utils;
use serde_json;

// use std::io::prelude::*;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MemInit {
    pub base : u16,
    pub size : usize,
    pub writeable : bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct RunLog {
    pub file_name : String,
    pub load_addr : u16,
    pub memory : Vec<MemInit>,
    pub states : Vec<Step>,
}


////////////////////////////////////////////////////////////////////////////////

use crate::timer::{Timer, RunTime};


////////////////////////////////////////////////////////////////////////////////

impl RunLog {

    pub fn create_memmap(&self) -> MemMap {

        let full_file_name = format!("cpp/{}", self.file_name);

        let mut m = MemMap::new();

        use crate::utils::{load_file};

        for mb in &self.memory {
            m.add_mem_block("NO NAME", mb.writeable, mb.base, mb.size)
        };

        let data = load_file(&full_file_name);
        let addr = self.load_addr;
        m.upload(addr, &data);
        m
    }
}

pub struct JsonTest {

    check_cycles    : bool,
    verbose         : bool,
    dont_check_hash : bool,
    json_file       : String,
    log_memory      : bool,
    mem             : MemMap,
    steps           : Vec<Step>,
    regs            : Regs,
    clock           : Rc<RefCell<StandardClock>>,
}


fn load_json(json_file : &str, loader : fn(&str) -> RunLog) -> (RunTime, RunLog) {
    let mut timer = Timer::new();
    let run_log = loader(json_file);
    let dur =timer.get();
    (dur, run_log)
}

fn time_func<T>( func : &dyn Fn() -> T) -> (RunTime, T) {
    let mut timer = Timer::new();
    let ret =  func();
    (timer.get(), ret)
}

impl tester::Tester for JsonTest {

    fn from_matches( matches : &ArgMatches ) -> JsonTest {

        let json_file = matches.value_of("JSON FILE").unwrap().to_string();

        // let buffer_loader = || -> RunLog {
        //     let br = BufReader::new(File::open(&json_file).unwrap());
        //     serde_json::from_reader(br).unwrap()
        // };

        let string_loader = || -> RunLog {
            let mut json_contents = String::new();
            File::open(&json_file).unwrap().read_to_string(&mut json_contents).unwrap();
            serde_json::from_str(&json_contents).unwrap()
        };

        let (dur, run_log) = time_func(&string_loader);
        println!("string loader: {} seconds", dur.secs());

        // let (dur, run_log) = time_func(&buffer_loader);
        // println!("buffer loader: {} seconds", dur.secs());

        println!("Done, {} steps to emulate", run_log.states.len().separated_string());

        let rc_clock = Rc::new(RefCell::new(StandardClock::new(1_500_000)));

        let start_regs = run_log.states[0].regs.clone();

        JsonTest {
            json_file       : json_file.clone(),
            dont_check_hash : matches.is_present("no-hash-check"),
            log_memory      : matches.is_present("log-memory"),
            mem             : run_log.create_memmap(),
            // cpu             : Cpu::from_regs(&start_regs),
            steps           : run_log.states,
            check_cycles    : matches.is_present("check-cycles"),
            verbose         : matches.is_present("show-disassembly"),
            regs            : start_regs.clone(),
            clock           : rc_clock,
        }
    }

    fn run(&mut self) {

        println!("Comparing to test run in {}", self.json_file);
        println!("Skipping hash check: {}", self.dont_check_hash);

        // let base_mem = run_log.create_memmap();
        // let mut mem = LoggingMemMap::new(base_mem);


        let mut diss = Disassembler::new();

        let mut it = self.steps.iter().peekable();

        let mut timer = Timer::new();

        for i in 0 .. self.steps.len()/2 {

            // mem.clear_log();

            let log_before = &it.next().unwrap();
            let log_after = &it.peek().unwrap();
            let log_cycles = log_after.cycles - log_before.cycles;


            let log_regs_after = &log_after.regs;

            let prev_sim = self.regs.clone();

            let pc = self.regs.pc;

            let ins = step(&mut self.regs, &mut self.mem, &self.clock);

            if let Ok(ins) = ins {

            let sim = &self.regs;

            let is_hash_ok = if self.dont_check_hash {
                true
            } else {
                match log_after.digest {
                    Some(ref d) => {
                        let hash = self.mem.get_sha1_string();
                        hash == *d
                    }
                    _ => true,
                }
            };

            if self.verbose {
                let (_, txt) =  diss.diss(&mut self.mem, pc, None);
                println!("({:5}) : ${:04x}   {:20} : {} ", ins.cycles, pc, txt, sim);
            }


            let are_cycles_okay = !self.check_cycles || (ins.cycles == log_cycles as u32);


            if ( sim != log_regs_after ) | !is_hash_ok  | !are_cycles_okay {

                if !are_cycles_okay {
                    println!("Cycles Error at ${:02x} is {} should be {}",pc,  ins.cycles, log_cycles);
                }

                println!("Error after {} instructions", i);

                // let writes_str = get_writes_as_str(&mem);
                // println!("{:04x}   {:20}{:20} : {}", pc, txt, writes_str, sim);

                let (_, txt) =  diss.diss(&mut self.mem, self.regs.pc, None);
                println!();

                println!("Next op:");
                println!("{:04x}   {:20}", self.regs.pc, txt);

                println!();

                if !is_hash_ok {
                    let hash = self.mem.get_sha1_string();
                    // let log_hash = state_after.digest.unwrap().clone();
                    println!("       sim: {}", hash);
                    // println!(" should be: {}", log_hash);
                }

                println!();

                println!("            {}", Regs::get_hdr());
                println!("      prev: {}", prev_sim);
                println!("       sim: {}", sim);
                println!(" should be: {}", log_regs_after);

                println!();

                panic!("Done");

            } 
            }
            

            // let ins = self.cpu.step(&mut self.mem);


        }

        let ins = self.steps.len();
        println!("Successfully run {} instructions", ins.separated_string());
        report(&timer.get(), ins);

        println!();
    }
}


fn get_writes_as_str( mem : &LoggingMemMap ) -> String {

    let writes : Vec<LogEntry>= mem.get_log()
        .into_iter()
        .filter(|msg| msg.write)
        .collect();

    if writes.is_empty() {
        "".to_string()
    } else {
        writes[0].to_string()
    }
}


fn report(runtime : &RunTime, ins : usize)  {

    let secs = runtime.secs();

    let ins_per_second = ins as f64 / secs;

    let cycles_per_instruction = 4;
    let cycles_per_instructionf = f64::from(cycles_per_instruction);

    println!("instructions: {}", ins.separated_string());
    println!("secs:         {:.4}", secs);

    println!("{} instructions per second", ( ins_per_second as u32 ).separated_string());
    println!("{:0.02}mhz (est avg {} cycler per instruction)"
             , (ins_per_second * cycles_per_instructionf) / 1_000_000.0
             , cycles_per_instruction
            );
}

