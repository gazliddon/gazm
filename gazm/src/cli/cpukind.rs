use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Default)]
pub enum CpuKind {
    #[default]
    Cpu6809,
    Cpu6800,
    Cpu6502,
    Cpu65c02,
    CpuZ80,
}

impl TryFrom<&str> for CpuKind {
    type Error = ();
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        use CpuKind::*;
        let input = input.to_owned().to_lowercase();
        match input.as_str() {
            "6800" => Ok(Cpu6800),
            "6502" => Ok(Cpu6809),
            "65c02" => Ok(Cpu6809),
            "z80" => Ok(CpuZ80),
            "6809" => Ok(Cpu6809),
            _ => panic!()
        }
    }
}
