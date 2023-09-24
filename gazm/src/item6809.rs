use std::arch::aarch64::vld2_lane_p8;
use std::fmt::Display;
use std::{collections::HashSet, path::PathBuf};
use thin_vec::ThinVec;

use crate::{
    ast::AstNodeId,
    error::ParseError,
    item::Item,
    node::{BaseNode, CtxTrait},
    parse::locate::span_to_pos,
    parse::locate::Span,
};

use emu6809::{
    cpu::{IndexedFlags, RegEnum},
    isa::Instruction,
};

use sources::Position;


use crate::symbols::SymbolScopeId;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexParseType {
    ConstantOffset(RegEnum), //             arg,R
    Plus(RegEnum),           //             ,R+                    2 0 |
    PlusPlus(RegEnum),       //             ,R++                   3 0 |
    Sub(RegEnum),            //             ,-R                    2 0 |
    SubSub(RegEnum),         //             ,--R                   3 0 |
    Zero(RegEnum),           //             ,R                     0 0 |
    AddB(RegEnum),           //             (+/- B),R              1 0 |
    AddA(RegEnum),           //             (+/- A),R              1 0 |
    AddD(RegEnum),           //             (+/- D),R              4 0 |
    PCOffset,                //             (+/- 7 bit offset),PC  1 1 |
    ExtendedIndirect,        //  [expr]
    Constant5BitOffset(RegEnum, i8),
    ConstantByteOffset(RegEnum, i8),
    ConstantWordOffset(RegEnum, i16),
    PcOffsetWord(i16),
    PcOffsetByte(i8),
}

impl IndexParseType {
    pub fn has_operand(&self) -> bool {
        use IndexParseType::*;

        match self {
            ConstantOffset(..) => true, //             arg,R
            Plus(..) => false,          //             ,R+                    2 0 |
            PlusPlus(..) => false,      //             ,R++                   3 0 |
            Sub(..) => false,           //             ,-R                    2 0 |
            SubSub(..) => false,        //             ,--R                   3 0 |
            Zero(..) => false,          //             ,R                     0 0 |
            AddB(..) => false,          //             (+/- B),R              1 0 |
            AddA(..) => false,          //             (+/- A),R              1 0 |
            AddD(..) => false,          //             (+/- D),R              4 0 |
            PCOffset => true,           //             (+/- 7 bit offset),PC  1 1 |
            ExtendedIndirect => true,   //  [expr]
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
            Plus(r) => {
                let mut bits = 0b1000_0000;
                bits = add_reg(bits, r);
                bits
            }

            PlusPlus(r) => {
                let mut bits = 0b1000_0001;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            Sub(r) => {
                let mut bits = 0b1000_0010;
                bits = add_reg(bits, r);
                bits
            }

            SubSub(r) => {
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

impl From<MC6809> for Item {
    fn from(value: MC6809) -> Self {
        Item::Cpu(value)
    }
}

// TODO: Ultimately this contains all of the CPU dependent AST node items
#[derive(Debug, PartialEq, Clone)]
pub enum MC6809 {
    SetDp,
    OpCode(String, Box<Instruction>, AddrModeParseType),
    Operand(AddrModeParseType),
    OperandIndexed(IndexParseType, bool),
    RegisterSet(HashSet<RegEnum>),
}

impl MC6809 {
    pub fn operand_from_index_mode(imode: IndexParseType, indirect: bool) -> Item {
        MC6809::OperandIndexed(imode, indirect).into()
    }
}
