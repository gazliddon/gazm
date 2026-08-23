use crate::{assembler::AssemblerCpuTrait, cpu6800::Asm6800, cpu6809::Asm6809};
use serde::Deserialize;

use strum_macros::EnumString;

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Default, EnumString, Eq)]
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
            CpuKind::Cpu6800 => Box::new(Asm6800::new()),
            CpuKind::Cpu6502 => todo!(),
            CpuKind::Cpu65c02 => todo!(),
            CpuKind::CpuZ80 => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CpuKind;
    use crate::assembler::AssemblerCpuTrait;

    #[test]
    fn cpu_kind_selects_matching_backend() {
        let cpu6809: Box<dyn AssemblerCpuTrait> = CpuKind::Cpu6809.into();
        let cpu6800: Box<dyn AssemblerCpuTrait> = CpuKind::Cpu6800.into();

        assert_eq!(cpu6809.get_cpu_name(), "6809");
        assert_eq!(cpu6800.get_cpu_name(), "6800");
    }
}
