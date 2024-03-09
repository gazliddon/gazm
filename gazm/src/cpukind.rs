use std::usize;

use crate::{assembler::AssemblerCpuTrait, cpu6800::Asm6800, cpu6809::Asm6809};
use emu6800::emucore::sha1::digest::DynDigest;
use serde::Deserialize;
use strum::{EnumCount, IntoEnumIterator, };

use strum_macros::{EnumCount as EnumCountMacro, EnumIter, EnumString};

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Default, EnumCountMacro, EnumIter, EnumString, Eq)]
#[repr(usize)]
pub enum CpuKind {
    #[default]
    Cpu6809,
    Cpu6800,
    Cpu6502,
    Cpu65c02,
    CpuZ80,
}

impl From<CpuKind> for Box<dyn AssemblerCpuTrait> {
    fn from(cpu: CpuKind) -> Box<dyn AssemblerCpuTrait> {
        match cpu {
            CpuKind::Cpu6809 => Box::new(Asm6809::new()),
            CpuKind::Cpu6800 => Box::new(Asm6809::new()),
            CpuKind::Cpu6502 => todo!(),
            CpuKind::Cpu65c02 => todo!(),
            CpuKind::CpuZ80 => todo!(),
        }
    }
}

pub struct Assemblers {
    asm: [Box<dyn AssemblerCpuTrait>; CpuKind::COUNT],
}

impl Default for Assemblers {
    fn default() -> Self {
        let res: Vec<Box<dyn AssemblerCpuTrait>> = CpuKind::iter().map(|x| x.into()).collect();

        Self {
            asm: res.try_into().unwrap_or_else(|_| panic!()),
        }
    }
}

impl Assemblers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn asm(&self, cpu: CpuKind) -> &dyn AssemblerCpuTrait {
        self.asm.get(cpu as usize).unwrap().as_ref()
    }
    pub fn asm_mut(&mut self, cpu: CpuKind) -> &mut dyn AssemblerCpuTrait {
        self.asm.get_mut(cpu as usize).unwrap().as_mut()
    }
}

pub struct CpuAssmbler {
    cpu: Option<CpuKind>,
    assemblers: Assemblers,
}

impl Default for CpuAssmbler {
    fn default() -> Self {
        Self {
            cpu: None,
            assemblers: Assemblers::new(),
        }
    }
}

impl CpuAssmbler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn asm(&self) -> &dyn AssemblerCpuTrait {
        let cpu = self.cpu.unwrap();
        let ret = self.assemblers.asm(cpu);
        ret
    }

    pub fn asm_mut(&mut self) -> &mut dyn AssemblerCpuTrait {
        let cpu = self.cpu.unwrap();
        let ret = self.assemblers.asm_mut(cpu);
        ret
    }
}

