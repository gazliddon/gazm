use std::fmt::Display;
use std::{collections::HashSet, path::PathBuf};

use crate::locate::span_to_pos;
use crate::locate::Span;
use crate::macros::MacroCall;
use crate::node::{BaseNode, CtxTrait};
use emu::cpu::{IndexedFlags, RegEnum};
use emu::isa::Instruction;
use utils::sources::Position;

impl<'a> CtxTrait for Span<'a> {}

pub type Node = BaseNode<Item, Position>;

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
                let mut bits = 0b10000100;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddA(r) => {
                let mut bits = 0b10000110;
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

#[derive(Debug, PartialEq, Clone)]
pub enum StructMemberType {
    Byte,
    Word,
    DWord,
    QWord,
    UserType(String),
}

impl StructMemberType {
    pub fn to_size_item(&self) -> Item {
        use Item::*;
        match self {
            Self::Byte => Number(1),
            Self::Word => Number(2),
            Self::DWord => Number(4),
            Self::QWord => Number(8),
            Self::UserType(name) => Label(format!("{}.size", name)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructEntry {
    pub name: String,
    pub item_type: StructMemberType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Skip(usize),
    LocalAssignment(String),
    Assignment(String),
    AssignmentFromPc(String),
    LocalAssignmentFromPc(String),

    MacroCall(MacroCall),
    // MacroDef(MacroDef),
    StructDef(String),
    StructEntry(String),

    SetPc(u16),
    SetPutOffset(isize),
    Scope(String),

    Expr,
    PostFixExpr,
    BracketedExpr,
    Pc,
    Block,
    ExpandedMacro(MacroCall),

    UnaryTerm,

    RegisterSet(HashSet<RegEnum>),

    Label(String),
    LocalLabel(String),
    Comment(String),
    Number(i64),

    OpCode(Instruction, AddrModeParseType),
    Operand(AddrModeParseType),
    OperandIndexed(IndexParseType, bool),
    Include(PathBuf),
    IncBin(PathBuf),
    IncBinRef(PathBuf),
    GrabMem,
    IncBinResolved {
        file: PathBuf,
        r: std::ops::Range<usize>,
    },

    WriteBin(PathBuf),

    TokenizedFile(PathBuf, PathBuf),

    Org,
    Put,

    Fdb(usize),
    Fcb(usize),
    Fcc(String),

    Rmb,
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
    ShiftRight,
    ShiftLeft,
    UnaryGreaterThan,
}

impl Item {
    pub fn operand_from_index_mode(imode: IndexParseType, indirect: bool) -> Self {
        Self::OperandIndexed(imode, indirect)
    }

    // pub fn is_empty_comment(&self) -> bool {
    //     if let Item::Comment(com) = &*self {
    //         com.is_empty()
    //     } else {
    //         false
    //     }
    // }

    // pub fn zero() -> Self {
    //     Self::number(0)
    // }

    pub fn number(n: i64) -> Self {
        Item::Number(n)
    }

    // pub fn get_my_tokenized_file(&self) -> Option<(&PathBuf, &PathBuf)> {
    //     if let Item::TokenizedFile(file, parent) = self {
    //         Some((file, parent))
    //     } else {
    //         None
    //     }
    // }

    // pub fn am_i_tokenized_file(&self) -> bool {
    //     self.get_my_tokenized_file().is_some()
    // }

    // pub fn is_tokenized_file(i: &Item) -> bool {
    //     i.am_i_tokenized_file()
    // }

    // pub fn get_tokenized_file(i: &Item) -> Option<(&PathBuf, &PathBuf)> {
    //     i.get_my_tokenized_file()
    // }

    pub fn get_number(&self) -> Option<i64> {
        if let Item::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }
    // pub fn label_name(&self) -> Option<&String> {
    //     if let Item::Label(n) = self {
    //         Some(n)
    //     } else {
    //         None
    //     }
    // }
}

impl BaseNode<Item, Position> {
    // pub fn is_empty_comment(&self) -> bool {
    //     match self.item() {
    //         Item::Comment(text) => text.is_empty(),
    //         _ => false,
    //     }
    // }
    pub fn from_item_pos(item: Item, p: Position) -> Self {
        Self::new(item, vec![], p)
    }

    pub fn from_item_span(item: Item, sp: Span) -> Self {
        Self::new(item, vec![], span_to_pos(sp))
    }

    pub fn from_number(n: i64, sp: Span) -> Self {
        Self::from_item_span(Item::Number(n), sp)
    }

    // pub fn to_label(txt: &str, ctx: Position) -> Self {
    //     Self::from_item(Item::Label(txt.to_string()), ctx)
    // }
    // pub fn to_local_label(txt: &str, ctx: Position) -> Self {
    //     Self::from_item(Item::LocalLabel(txt.to_string()), ctx)
    // }

    // pub fn get_label_name(&self) -> Option<&String> {
    //     if let Item::Label(name) = self.item() {
    //         Some(name)
    //     } else {
    //         None
    //     }
    // }

    // pub fn get_include_file(&self) -> Option<&PathBuf> {
    //     match self.item() {
    //         Item::Include(name) => Some(name),
    //         _ => None,
    //     }
    // }

    // pub fn is_include_file(&self) -> bool {
    //     self.get_include_file().is_some()
    // }
    pub fn with_span(self, sp: Span) -> Self {
        let mut ret = self;
        ret.ctx = span_to_pos(sp);
        ret
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
            // QuotedString(test) => format!("\"{}\"", test),
            // Register(r) => r.to_string(),

            // RegisterList(vec) => join_vec(vec, ","),
            LocalAssignment(name) | Assignment(name) => {
                format!("{} equ {}", name, self.children[0])
            }

            Expr => join_children(""),

            Include(file) => format!("include \"{}\"", file.to_string_lossy()),

            Number(n) => n.to_string(),
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

            OpCode(ins, addr_type) => {
                format!("{} {:?}", ins.action, addr_type)
            }

            TokenizedFile(file, _) => {
                let header = format!("; included file {}", file.to_string_lossy());
                let children: Vec<String> =
                    self.children.iter().map(|n| format!("{}", &*n)).collect();
                format!("{}\n{}", header, children.join("\n"))
            }

            Block => {
                let children: Vec<String> =
                    self.children.iter().map(|n| format!("{}", &*n)).collect();
                children.join("\n")
            }

            SetDp => {
                let children: Vec<String> =
                    self.children.iter().map(|n| format!("{}", &*n)).collect();
                children.join("\n")
            }

            _ => format!("{:?} not implemented", item),
        };

        write!(f, "{}", ret)
    }
}
