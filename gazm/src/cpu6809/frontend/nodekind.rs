#![forbid(unused_imports)]
use crate::{cpu6809::NodeKind, frontend::CpuSpecific};
use std::collections::HashSet;

use emu6809::{
    cpu::{IndexedFlags, RegEnum},
    isa::InstructionId,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexParseType {
    ConstantOffset(RegEnum, IndexWidth), // arg,R
    PostInc(RegEnum),                    // ,R+                    2 0 |
    PostIncInc(RegEnum),                 // ,R++                   3 0 |
    PreDec(RegEnum),                     // ,-R                    2 0 |
    PreDecDec(RegEnum),                  // ,--R                   3 0 |
    Zero(RegEnum),                       // ,R                     0 0 |
    AddB(RegEnum),                       // (+/- B),R              1 0 |
    AddA(RegEnum),                       // (+/- A),R              1 0 |
    AddD(RegEnum),                       // (+/- D),R              4 0 |
    PCOffset,                            // (+/- 7 bit offset),PC  1 1 |
    ExtendedIndirect,                    //  [expr]
    Constant5BitOffset(RegEnum, i8),
    ConstantByteOffset(RegEnum, i8),
    ConstantWordOffset(RegEnum, i16),
    PcOffsetWord(i16),
    PcOffsetByte(i8),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexWidth {
    Auto,
    Bits5,
    Byte,
    Word,
}

/// The indexed postbyte: a base value per form with the register
/// (bits 6-5) and indirect (bit 4) fields ORed in. Every base starts
/// with those fields clear, so no masking is needed.
impl IndexParseType {
    pub fn get_index_byte(&self, indirect: bool) -> u8 {
        use IndexParseType::*;

        let ind = if indirect {
            IndexedFlags::IND.bits()
        } else {
            0
        };
        let reg = |r: RegEnum| rbits(r) | ind;

        match *self {
            PostInc(r) => 0b1000_0000 | rbits(r),
            PostIncInc(r) => 0b1000_0001 | reg(r),
            PreDec(r) => 0b1000_0010 | rbits(r),
            PreDecDec(r) => 0b1000_0011 | reg(r),
            Zero(r) => 0b1000_0100 | reg(r),
            AddA(r) => 0b1000_0110 | reg(r),
            AddB(r) => 0b1000_0101 | reg(r),
            AddD(r) => 0b1000_1011 | reg(r),
            PcOffsetByte(_) => 0b1000_1100 | ind,
            PcOffsetWord(_) => 0b1000_1101 | ind,
            ExtendedIndirect => 0b1001_1111,
            Constant5BitOffset(r, off) => reg(r) | (off as u8 & 0x1f),
            ConstantByteOffset(r, _) => 0b1000_1000 | reg(r),
            ConstantWordOffset(r, _) => 0b1000_1001 | reg(r),
            PCOffset | ConstantOffset(..) => panic!("Internal error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use IndexParseType::*;

    /// The postbyte table: base per form, register bits 6-5, indirect
    /// bit 4. A regression test — the register shift was once dropped
    /// and every indexed form encoded with the X register.
    #[test]
    fn indexed_postbytes() {
        use RegEnum::*;
        let cases = [
            (PostInc(X), false, 0x80),
            (PostIncInc(Y), true, 0x81 | 0x20 | 0x10),
            (PreDec(U), false, 0x82 | 0x40),
            (PreDecDec(S), true, 0x83 | 0x60 | 0x10),
            (Zero(X), false, 0x84),
            (Zero(Y), true, 0x84 | 0x20 | 0x10),
            (AddA(S), true, 0x86 | 0x60 | 0x10),
            (AddB(Y), false, 0x85 | 0x20),
            (AddD(U), false, 0x8b | 0x40),
            (PcOffsetByte(0), true, 0x8c | 0x10),
            (PcOffsetWord(0), false, 0x8d),
            (ExtendedIndirect, false, 0x9f),
            (Constant5BitOffset(Y, -3), false, 0x20 | 0x1d),
            (ConstantByteOffset(Y, 8), true, 0x88 | 0x20 | 0x10),
            (ConstantWordOffset(Y, 300), true, 0x89 | 0x20 | 0x10),
        ];
        for (mode, indirect, expected) in cases {
            assert_eq!(
                mode.get_index_byte(indirect),
                expected,
                "{mode:?} indirect={indirect}"
            );
        }
    }
}

/// Register bits for the indexed postbyte (bits 6-5: X=0, Y=1, U=2, S=3).
fn rbits(r: RegEnum) -> u8 {
    match r {
        RegEnum::X => 0,
        RegEnum::Y => 1 << 5,
        RegEnum::U => 2 << 5,
        RegEnum::S => 3 << 5,
        _ => panic!("internal error"),
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AddrModeParseType {
    Indexed(IndexParseType, bool),
    Direct,
    Extended(bool), // if set then extended mode was forced, do not opt for DP
    Relative,
    Inherent,
    Immediate,
    RegisterSet,
    RegisterPair(RegEnum, RegEnum),
}

impl From<AddrModeParseType> for NodeKind {
    fn from(value: AddrModeParseType) -> Self {
        NodeKind::TargetSpecific(NodeKind6809::Operand(value).into())
    }
}

impl From<NodeKind6809> for NodeKind {
    fn from(value: NodeKind6809) -> Self {
        NodeKind::TargetSpecific(value.into())
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum NodeKind6809 {
    #[default]
    Illegal,
    SetDp,
    OpCode(InstructionId, AddrModeParseType),
    Operand(AddrModeParseType),
    OperandIndexed(IndexParseType, bool),
    RegisterSet(HashSet<RegEnum>),
}

impl From<NodeKind6809> for CpuSpecific {
    fn from(value: NodeKind6809) -> Self {
        CpuSpecific::Cpu6809(value)
    }
}

impl NodeKind6809 {
    pub fn operand_from_index_mode(imode: IndexParseType, indirect: bool) -> NodeKind {
        NodeKind6809::OperandIndexed(imode, indirect).into()
    }
}
