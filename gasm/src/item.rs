use std::fmt::Display;
use std::ops::Add;
use std::path::Path;
use std::{collections::HashSet, path::PathBuf, slice::Iter};

use emu::cpu::{IndexedFlags, RegEnum};
use emu::isa::{AddrModeEnum, Instruction, InstructionInfo};
use nom::{IResult, Offset};

use crate::ctx::Ctx;
use crate::fileloader::FileLoader;
use crate::locate::{matched_span, Span};
use crate::node::{BaseNode, CtxTrait};

use crate::locate::Position;
use crate::postfix::GetPriotity;

impl<'a> CtxTrait for Span<'a> {}

pub type Node = BaseNode<Item, Position>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexParseType {
    ConstantOffset(RegEnum, bool), //               arg,R

    Plus(RegEnum),           //               ,R+              2 0 |
    PlusPlus(RegEnum, bool), //               ,R++             3 0 |
    Sub(RegEnum),            //               ,-R              2 0 |
    SubSub(RegEnum, bool),   //               ,--R             3 0 |
    Zero(RegEnum, bool),     //               ,R               0 0 |
    AddB(RegEnum, bool),     //             (+/- B),R          1 0 |
    AddA(RegEnum, bool),     //             (+/- A),R          1 0 |
    AddD(RegEnum, bool),     //             (+/- D),R          4 0 |
    PCOffset(bool),          //      (+/- 7 bit offset),PC     1 1 |
    ExtendedIndirect,        //  [expr]
    Constant5BitOffset(RegEnum, i8, bool),
    ConstantByteOffset(RegEnum, i8, bool),
    ConstantWordOffset(RegEnum, i16, bool),
    PcOffsetWord(i16, bool),
    PcOffsetByte(i8, bool),
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
    pub fn get_index_byte(&self) -> u8 {
        use IndexParseType::*;

        match *self {
            Plus(r) => {
                let mut bits = 0b1000_0000;
                bits = add_reg(bits, r);
                bits
            }

            PlusPlus(r, indirect) => {
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

            SubSub(r, indirect) => {
                let mut bits = 0b1000_0011;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            Zero(r, indirect) => {
                let mut bits = 0b10000100;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddA(r, indirect) => {
                let mut bits = 0b10000110;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddB(r, indirect) => {
                let mut bits = 0b1000_0101;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddD(r, indirect) => {
                let mut bits = 0b1000_1011;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            PcOffsetByte(_, indirect) => {
                let mut bits = 0b1000_1100;
                bits = add_ind(bits, indirect);
                bits
            }

            PcOffsetWord(_, indirect) => {
                let mut bits = 0b1000_1101;
                bits = add_ind(bits, indirect);
                bits
            }

            ExtendedIndirect => {
                0b1001_1111
            },

            Constant5BitOffset(r, off, indirect) => {
                let mut bits = 0b0000_0000;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits  = bits | (off as u8 &0x1f);
                bits
            }

            ConstantByteOffset(r, _, indirect) => {
                let mut bits = 0b1000_1100;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }
            ConstantWordOffset(r, _, indirect) => {
                let mut bits = 0b1000_1001;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            PCOffset(..) |
            ConstantOffset(..) => panic!("Internal error"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AddrModeParseType {
    Indexed(IndexParseType),
    Direct,
    Extended,
    Relative,
    Inherent,
    Immediate,
    RegisterSet,
}

impl AddrModeParseType {
    pub fn get_instruction<'a>(&self, info: &'a InstructionInfo) -> Option<&'a Instruction> {
        use AddrModeEnum::*;
        let get = |amode| info.get_instruction(&amode);

        match self {
            Self::Indexed(_) => get(Indexed),

            Self::Direct => get(Direct),

            Self::Extended => get(Extended)
                .or_else(|| get(Relative))
                .or_else(|| get(Relative16)),

            Self::Relative => get(Relative).or_else(|| get(Relative16)),

            Self::Inherent => get(Inherent),

            Self::Immediate => get(Immediate8).or_else(|| get(Immediate16)),

            Self::RegisterSet => get(RegisterSet),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    LocalAssignment(String),
    Assignment(String),
    AssignmentFromPc(String),
    LocalAssignmentFromPc(String),

    SetPc(u16),

    Expr,
    PostFixExpr,
    BracketedExpr,
    Pc,
    Block,

    UnaryOp,
    UnaryTerm,

    RegisterList(Vec<RegEnum>),
    RegisterSet(HashSet<RegEnum>),

    Label(String),
    LocalLabel(String),
    Comment(String),
    QuotedString(String),
    Number(i64),

    OpCode(Instruction, AddrModeParseType),
    Operand(AddrModeParseType),

    Register(RegEnum),

    Include(PathBuf),

    TokenizedFile(PathBuf, PathBuf, String),

    Org,
    Fdb(usize),
    Fcb(usize),
    Fill,
    Zmb,
    Zmd,
    SetDp,

    Mul,
    Div,
    Add,
    Sub,
    And,
    Or,
    Xor,
    UnaryPlus,
    UnaryMinus,
}

impl GetPriotity for Item {
    fn priority(&self) -> Option<usize> {
        match self {
            Item::Mul => Some(5),
            Item::Div => Some(5),
            Item::Add => Some(4),
            Item::Sub => Some(4),
            _ => None,
        }
    }
}

impl Item {
    pub fn operand_from_index_mode(imode: IndexParseType) -> Self {
        Self::Operand(AddrModeParseType::Indexed(imode))
    }

    pub fn is_empty_comment(&self) -> bool {
        if let Item::Comment(com) = &*self {
            com.is_empty()
        } else {
            false
        }
    }

    pub fn zero() -> Self {
        Self::number(0)
    }

    pub fn number(n: i64) -> Self {
        Item::Number(n)
    }

    pub fn get_my_tokenized_file(&self) -> Option<(&PathBuf, &PathBuf, &String)> {
        if let Item::TokenizedFile(file, parent, source) = self {
            Some((file, parent, source))
        } else {
            None
        }
    }

    pub fn am_i_tokenized_file(&self) -> bool {
        self.get_my_tokenized_file().is_some()
    }

    pub fn is_tokenized_file(i: &Item) -> bool {
        i.am_i_tokenized_file()
    }

    pub fn get_tokenized_file(i: &Item) -> Option<(&PathBuf, &PathBuf, &String)> {
        i.get_my_tokenized_file()
    }

    pub fn get_number(&self) -> Option<i64> {
        if let Item::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }
    pub fn label_name(&self) -> Option<&String> {
        if let Item::Label(n) = self {
            Some(n)
        } else {
            None
        }
    }
}

impl<E: CtxTrait> BaseNode<Item, E> {
    pub fn is_empty_comment(&self) -> bool {
        match self.item() {
            Item::Comment(text) => text.is_empty(),
            _ => false,
        }
    }

    pub fn from_number<X>(n: i64, ctx: X) -> Self
    where
        X: Into<E>,
    {
        Self::from_item(Item::Number(n), ctx.into())
    }

    pub fn to_label(txt: &str, ctx: E) -> Self {
        Self::from_item(Item::Label(txt.to_string()), ctx)
    }
    pub fn to_local_label(txt: &str, ctx: E) -> Self {
        Self::from_item(Item::LocalLabel(txt.to_string()), ctx)
    }

    pub fn get_label_name(&self) -> Option<&String> {
        if let Item::Label(name) = self.item() {
            Some(name)
        } else {
            None
        }
    }

    pub fn get_include_file(&self) -> Option<&PathBuf> {
        match self.item() {
            Item::Include(name) => Some(name),
            _ => None,
        }
    }

    pub fn is_include_file(&self) -> bool {
        self.get_include_file().is_some()
    }
}

pub fn join_vec<I: Display>(v: &[I], sep: &str) -> String {
    let ret: Vec<_> = v.iter().map(|x| x.to_string()).collect();
    ret.join(sep)
}

impl<'a> Display for BaseNode<Item, Position> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Item::*;

        let item = self.item();

        let join_children = |sep| join_vec(&self.children, sep);

        let ret: String = match item {
            LocalAssignmentFromPc(name) | AssignmentFromPc(name) => {
                format!("{} equ {}", name, self.children[0])
            }
            Pc => "*".to_string(),

            Label(name) | LocalLabel(name) => name.clone(),

            Comment(comment) => comment.clone(),
            QuotedString(test) => format!("\"{}\"", test),
            Register(r) => r.to_string(),

            RegisterList(vec) => join_vec(vec, ","),

            LocalAssignment(name) | Assignment(name) => {
                format!("{} equ {}", name, self.children[0])
            }

            Expr => join_children(""),

            Include(file) => format!("include \"{}\"", file.to_string_lossy()),

            Number(n) => n.to_string(),
            UnaryMinus => "-".to_string(),
            UnaryTerm => join_children(""),

            Mul => "*".to_string(),
            Div => "/".to_string(),
            Add => "+".to_string(),
            Sub => "-".to_string(),
            And => "&".to_string(),
            Or => "|".to_string(),
            Xor => "^".to_string(),
            Org => {
                format!("org {}", self.children[0])
            }

            BracketedExpr => {
                format!("({})", join_children(""))
            }

            TokenizedFile(file, _, _) => {
                let header = format!("; included file {}", file.to_string_lossy());
                let children: Vec<String> =
                    self.children.iter().map(|n| format!("{}", &*n)).collect();
                format!("{}\n{}", header, children.join("\n"))
            }

            _ => format!("{:?} not implemented", item),
        };

        write!(f, "{}", ret)
    }
}
