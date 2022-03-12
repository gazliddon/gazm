use super::mem::MemoryIO;

use byteorder::ByteOrder;
use op_table;

use super::cpu::{
    AddressLines, Direct, Extended, Immediate16, Immediate8, Indexed, Inherent, InstructionDecoder,
    RegisterPair, RegisterSet, Relative, Relative16,
};

use lru_cache::LruCache;

use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref CACHE : Mutex<LruCache<usize,Dissembly>> = {
        let ret = LruCache::new(10000);
        Mutex::new(ret)
    };
}

fn try_cache_or_create<F>(addr: usize, create: F) -> Dissembly
where
    F: Fn() -> Dissembly,
{
    let mut cache = CACHE.lock().unwrap();
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
    pub fn new(cache_size: usize) -> Self {
        Self {
            cache: LruCache::new(cache_size),
            previous_instruction: HashMap::new(),
        }
    }

    pub fn try_cache_or_create<E: ByteOrder, F>(&mut self, addr: u16, create: F) -> Dissembly
    where
        F: Fn() -> Dissembly,
    {
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
    pub addr: usize,
    pub next_instruction_addr: Option<usize>,
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
    pub mem: &'a mut dyn MemoryIO,
}

impl<'a> Disassembler<'a> {
    pub fn new(mem: &'a mut dyn MemoryIO) -> Self {
        Self { mem }
    }

    fn diss_op<A: AddressLines>(&self, _ins: &mut InstructionDecoder) -> String {
        let action = &_ins.instruction_info.action;
        format!("{:<5}{}", action, A::diss(self.mem, _ins))
    }

    pub fn diss(&mut self, _addr: usize) -> Dissembly {
        panic!()
    }
}
