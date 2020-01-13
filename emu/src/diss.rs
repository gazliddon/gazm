
use super::mem::{MemoryIO};

use super::cpu::{
    AddressLines, Direct, Extended, Immediate16, Immediate8, Indexed, Inherent,
    InstructionDecoder, Relative, Relative16
};

use lru_cache::LruCache;

use std::sync::Mutex;

lazy_static! {
    static ref CACHE : Mutex<LruCache<u16,Dissembly>> = {
        let ret = LruCache::new(10000);
        Mutex::new(ret)
    };
}

fn try_cache_or_create<F>( addr : u16, create : F) -> Dissembly where
F : Fn() -> Dissembly {
    let mut cache =  CACHE.lock().unwrap();
    if let Some(cached) = cache.get_mut(&addr).cloned() {
        cached 
    } else {
        let ret = create();
        cache.insert(addr, ret.clone());
        ret
    }
}

use std::collections::HashMap;

struct AssemblyCache {
    cache: LruCache<u16, Dissembly>,
    previous_instruction: HashMap<u16, u16>,
}

impl AssemblyCache {
    pub fn new(cache_size : usize) -> Self {
        Self {
            cache : LruCache::new(cache_size),
            previous_instruction: HashMap::new()
        }
    }

    pub fn try_cache_or_create<F>(&mut self, addr : u16, create : F) -> Dissembly where
        F : Fn() -> Dissembly {
            if let Some(cached) = self.cache.get_mut(&addr).cloned() {
                cached 
            } else {
                let ret = create();
                self.cache.insert(addr, ret.clone());
                ret
            }
        }
}



#[derive(Clone)]
pub struct Dissembly {
    pub text: String,
    pub addr: u16,
    pub next_instruction_addr: Option<u16>,
    instruction: InstructionDecoder,
}

impl Dissembly {
    pub fn is_illegal(&self) -> bool {
        self.instruction.instruction_info.action == "unknown"
    }
    pub fn is_legal(&self) -> bool {
        !self.is_illegal()
    }
}

pub struct Disassembler<'a> {
    pub mem: &'a dyn MemoryIO,
}

impl<'a> Disassembler<'a> {
    pub fn new(mem : &'a dyn MemoryIO) -> Self {
        Self {
            mem
        }
    }

    fn diss_op<A: AddressLines>(&self, _ins: &mut InstructionDecoder) -> String {
        let action =&_ins.instruction_info.action;
        format!("{:<5}{}", action, A::diss(self.mem, _ins))
    }

    pub fn diss(&self, addr : u16) -> Dissembly {

        try_cache_or_create(addr, || {

            let mut ins = InstructionDecoder::new_from_inspect_mem(addr, self.mem);

            macro_rules! handle_op {
                ($addr:ident, $action:ident, $opcode:expr, $cycles:expr, $size:expr) => {{
                    self.diss_op::<$addr>(&mut ins)
                }};
            }

            let text = op_table!(ins.instruction_info.opcode, { "".into() });

            // Now work out the address of the next instruction
            // None for adddresses that are invalid

            let next_instruction_addr = if self.mem.is_valid_addr(ins.next_addr) {
                Some(ins.next_addr)
            } else {
                None
            };

            Dissembly {
                text,
                addr,
                next_instruction_addr,
                instruction: ins,
            }
        })
    }
}
