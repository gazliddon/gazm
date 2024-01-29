mod assembler6800;
pub mod frontend;
pub mod assembler;

pub use assembler6800::*;

use emu6800::cpu_core::DBASE;

use emu6800::cpu_core::RegEnum;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AddrModeParseType {
    Indexed,
    Direct,
    Extended,
    Relative,
    Inherent,
    Immediate,
}

