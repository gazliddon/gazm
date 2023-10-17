use super::FrontEndError;
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CpuKind {
    Cpu6809,
    Cpu6800,
    Cpu6502,
    Cpu65c02,
    CpuZ80,
}

impl TryFrom<&str> for CpuKind {
    type Error = FrontEndError;
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        use CpuKind::*;

        match input {
            "6800" => Ok(Cpu6800),
            "6502" => Ok(Cpu6809),
            "65c02" => Ok(Cpu6809),
            "z80" => Ok(CpuZ80),
            "6809" => Ok(Cpu6809),
            _ => panic!()
        }
    }
}
