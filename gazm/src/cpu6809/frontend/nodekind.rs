#![forbid(unused_imports)]
use crate::{cpu6809::NodeKind, frontend::CpuSpecific};
use std::collections::HashSet;

use emu6809::{
    cpu::{IndexedFlags, RegEnum},
    isa::Instruction,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexParseType {
    ConstantOffset(RegEnum), // arg,R
    PostInc(RegEnum),        // ,R+                    2 0 |
    PostIncInc(RegEnum),     // ,R++                   3 0 |
    PreDec(RegEnum),         // ,-R                    2 0 |
    PreDecDec(RegEnum),      // ,--R                   3 0 |
    Zero(RegEnum),           // ,R                     0 0 |
    AddB(RegEnum),           // (+/- B),R              1 0 |
    AddA(RegEnum),           // (+/- A),R              1 0 |
    AddD(RegEnum),           // (+/- D),R              4 0 |
    PCOffset,                // (+/- 7 bit offset),PC  1 1 |
    ExtendedIndirect,        //  [expr]
    Constant5BitOffset(RegEnum, i8),
    ConstantByteOffset(RegEnum, i8),
    ConstantWordOffset(RegEnum, i16),
    PcOffsetWord(i16),
    PcOffsetByte(i8),
}

impl IndexParseType {
    pub fn allowed_indirect(&self) -> bool {
        use IndexParseType::*;
        match self {
            PostInc(..) => false, // ,R+                    2 0 |
            PreDec(..) => false,  // ,-R                    2 0 |
            _ => true,
        }
    }

    pub fn has_operand(&self) -> bool {
        use IndexParseType::*;

        match self {
            ConstantOffset(..) => true, // arg,R
            PostInc(..) => false,       // ,R+                    2 0 |
            PostIncInc(..) => false,    // ,R++                   3 0 |
            PreDec(..) => false,        // ,-R                    2 0 |
            PreDecDec(..) => false,     // ,--R                   3 0 |
            Zero(..) => false,          // ,R                     0 0 |
            AddB(..) => false,          // (+/- B),R              1 0 |
            AddA(..) => false,          // (+/- A),R              1 0 |
            AddD(..) => false,          // (+/- D),R              4 0 |
            PCOffset => true,           // (+/- 7 bit offset),PC  1 1 |
            ExtendedIndirect => true,   // [expr]
            Constant5BitOffset(..) => true,
            ConstantByteOffset(..) => true,
            ConstantWordOffset(..) => true,
            PcOffsetWord(..) => true,
            PcOffsetByte(..) => true,
        }
    }
}

fn rbits(r: RegEnum) -> u8 {
    let rnum = {
        match r {
            RegEnum::X => 0,
            RegEnum::Y => 1,
            RegEnum::U => 2,
            RegEnum::S => 3,
            _ => panic!("internal error"),
        }
    };

    rnum << 5
}

fn add_reg(bits: u8, r: RegEnum) -> u8 {
    (bits & !(3 << 5)) | rbits(r)
}

fn add_ind(bits: u8, ind: bool) -> u8 {
    let ind_bit = IndexedFlags::IND.bits();
    let ind_val = if ind { ind_bit } else { 0u8 };

    (bits & !ind_bit) | ind_val
}

impl IndexParseType {
    pub fn get_index_byte(&self, indirect: bool) -> u8 {
        use IndexParseType::*;

        match *self {
            PostInc(r) => {
                let mut bits = 0b1000_0000;
                bits = add_reg(bits, r);
                bits
            }

            PostIncInc(r) => {
                let mut bits = 0b1000_0001;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            PreDec(r) => {
                let mut bits = 0b1000_0010;
                bits = add_reg(bits, r);
                bits
            }

            PreDecDec(r) => {
                let mut bits = 0b1000_0011;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            Zero(r) => {
                let mut bits = 0b1000_0100;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddA(r) => {
                let mut bits = 0b1000_0110;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddB(r) => {
                let mut bits = 0b1000_0101;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddD(r) => {
                let mut bits = 0b1000_1011;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            PcOffsetByte(_) => {
                let mut bits = 0b1000_1100;
                bits = add_ind(bits, indirect);
                bits
            }

            PcOffsetWord(_) => {
                let mut bits = 0b1000_1101;
                bits = add_ind(bits, indirect);
                bits
            }

            ExtendedIndirect => 0b1001_1111,

            Constant5BitOffset(r, off) => {
                let mut bits = 0b0000_0000;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits |= off as u8 & 0x1f;
                bits
            }

            ConstantByteOffset(r, _) => {
                let mut bits = 0b1000_1000;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            ConstantWordOffset(r, _) => {
                let mut bits = 0b1000_1001;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            PCOffset | ConstantOffset(..) => panic!("Internal error"),
        }
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

impl AddrModeParseType {
    pub fn has_operand(&self) -> bool {
        use AddrModeParseType::*;

        match self {
            Direct => true,
            Extended(..) => true,
            Relative => true,
            Inherent => false,
            Immediate => true,
            RegisterSet => true,
            RegisterPair(..) => false,
            Indexed(x, _) => x.has_operand(),
        }
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
    OpCode(String, Box<Instruction>, AddrModeParseType),
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
