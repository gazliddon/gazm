mod assembler6800;
pub mod frontend;
pub mod assembler;

pub use assembler6800::*;

use emu6800::cpu_core::DBASE;

use emu6800::cpu_core::RegEnum;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AddrModeParseType {
    AccA,
    AccB,
    Indexed,
    Direct,
    Extended,
    Relative,
    Inherent,
    Immediate,
}

