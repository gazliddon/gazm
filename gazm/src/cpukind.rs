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

#[cfg(test)]
mod tests {
    use super::{CpuKind, Endianness};
    use crate::frontend::{lex_identifier, TokenKind};

    #[test]
    fn backend_registry_lexes_mnemonics() {
        // Each registered backend classifies its own mnemonics as opcodes
        // and everything else as a label.
        for (cpu, mnemonic) in [
            (CpuKind::Cpu6809, "LDX"),
            (CpuKind::Cpu6800, "LDAA"),
            (CpuKind::CpuZ80, "ldir"),
        ] {
            assert_eq!(
                lex_identifier(cpu, mnemonic),
                TokenKind::CpuOpcode(cpu),
                "{cpu:?} should recognise {mnemonic}"
            );
        }
        assert_eq!(lex_identifier(CpuKind::Cpu6809, "FOOBAR"), TokenKind::Label);
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
