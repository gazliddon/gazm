use crate::{assembler::AssemblerCpuTrait, cpu6800::Asm6800, cpu6809::Asm6809};
use serde::{Deserialize, Serialize};

use strum_macros::EnumString;

/// Byte order of a target's memory image. Governs multi-byte directive
/// writes (`write_word`/`write_long`/`write_quad`); single bytes are
/// unaffected. The 6800/6809/68000 family is big-endian; the 6502/65C02
/// and Z80 families are little-endian.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Big,
    Little,
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize, Default, EnumString, Eq, Hash)]
#[repr(usize)]
pub enum CpuKind {
    #[default]
    Cpu6809,
    Cpu6800,
    Cpu6502,
    Cpu65c02,
    CpuZ80,
    Cpu68000,
}

impl CpuKind {
    pub fn endianness(self) -> Endianness {
        use Endianness::*;
        match self {
            CpuKind::Cpu6809 | CpuKind::Cpu6800 | CpuKind::Cpu68000 => Big,
            CpuKind::Cpu6502 | CpuKind::Cpu65c02 | CpuKind::CpuZ80 => Little,
        }
    }
}

impl From<CpuKind> for Box<dyn AssemblerCpuTrait> {
    fn from(cpu: CpuKind) -> Box<dyn AssemblerCpuTrait> {
        match cpu {
            CpuKind::Cpu6809 => Box::new(Asm6809::new()),
            CpuKind::Cpu6800 => Box::new(Asm6800::new()),
            CpuKind::CpuZ80 => Box::new(crate::cpu_z80::AsmZ80::new()),
            CpuKind::Cpu6502 => todo!(),
            CpuKind::Cpu65c02 => todo!(),
            CpuKind::Cpu68000 => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CpuKind, Endianness};
    use crate::assembler::AssemblerCpuTrait;

    #[test]
    fn cpu_kind_selects_matching_backend() {
        let cpu6809: Box<dyn AssemblerCpuTrait> = CpuKind::Cpu6809.into();
        let cpu6800: Box<dyn AssemblerCpuTrait> = CpuKind::Cpu6800.into();

        assert_eq!(cpu6809.get_cpu_name(), "6809");
        assert_eq!(cpu6800.get_cpu_name(), "6800");
    }

    #[test]
    fn endianness_per_cpu() {
        use Endianness::*;
        assert_eq!(CpuKind::Cpu6809.endianness(), Big);
        assert_eq!(CpuKind::Cpu6800.endianness(), Big);
        assert_eq!(CpuKind::Cpu68000.endianness(), Big);
        assert_eq!(CpuKind::Cpu6502.endianness(), Little);
        assert_eq!(CpuKind::Cpu65c02.endianness(), Little);
        assert_eq!(CpuKind::CpuZ80.endianness(), Little);
    }
}
