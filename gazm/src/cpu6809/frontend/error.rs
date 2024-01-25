use thiserror::Error;

use super::AddrModeParseType;

// TODO Remove all of this, replace with help text
#[derive(Debug, Error, Clone, PartialEq, Copy)]
pub enum Cpu6809AssemblyErrorKind {
    #[error("This {0:?} is not supported for this opcode")]
    ThisAddrModeUnsupported(AddrModeParseType),
    #[error("Addressing mode is not supported for this opcode")]
    AddrModeUnsupported,
    #[error("This instruction only supports inherent mode addressing")]
    OnlySupports(AddrModeParseType),
}
