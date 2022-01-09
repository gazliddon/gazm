use std::fmt::Display;
use std::ops::Add;
use std::path::Path;
use std::{path::PathBuf, slice::Iter, collections::HashSet};

use emu::cpu::RegEnum;
use emu::isa::{AddrModeEnum, Instruction, InstructionInfo};
use nom::{IResult, Offset};

use crate::fileloader::FileLoader;
use crate::node::{BaseNode, CtxTrait};
use crate::ctx::Ctx;
use crate::locate::{Span, matched_span};

use crate::locate::Position;
use crate::postfix::GetPriotity;

impl<'a> CtxTrait for Span<'a> { }

pub type Node = BaseNode<Item, Position>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexParseType {
    ConstantOffset(RegEnum),    //               arg,R
    Plus(RegEnum),     //               ,R+              2 0 |
    PlusPlus(RegEnum), //               ,R++             3 0 |
    Sub(RegEnum),      //               ,-R              2 0 |
    SubSub(RegEnum),   //               ,--R             3 0 |
    Zero(RegEnum),     //               ,R               0 0 |
    AddB(RegEnum),     //             (+/- B),R          1 0 |
    AddA(RegEnum),     //             (+/- A),R          1 0 |
    AddD(RegEnum),     //             (+/- D),R          4 0 |
    PCOffset,          //      (+/- 7 bit offset),PC     1 1 |
    Indirect,          //  [expr]
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
    pub fn get_instruction<'a>(&self, info: &'a InstructionInfo) -> Option<&'a Instruction>{
        use AddrModeEnum::*;
        let get = |amode| info.get_instruction(&amode);

        match self {
            Self::Indexed(_)=>{get(Indexed)},

            Self::Direct=>{get(Direct)},

            Self::Extended=>{
                get(Extended)
                    .or_else(|| get(Relative))
                    .or_else(||get(Relative16))
            },

            Self::Relative=>{
                get(Relative)
                    .or_else(|| get(Relative16))
            },

            Self::Inherent=>{get(Inherent)},

            Self::Immediate=>{
                get(Immediate8)
                    .or_else(|| get(Immediate16))
            },

            Self::RegisterSet=>{get(RegisterSet)},
        }
    }
}




#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    LocalAssignment(String),
    Assignment(String),
    AssignmentFromPc(String),
    LocalAssignmentFromPc(String),

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
    Fdb,
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
            _=> None,
        }
    }
}

impl Item {
    pub fn operand_from_index_mode(imode : IndexParseType) -> Self {
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

    pub fn number(n : i64) -> Self {
        Item::Number(n)
    }

    pub fn get_my_tokenized_file(&self) -> Option<(&PathBuf, &PathBuf, &String)> {
        if let Item::TokenizedFile(file, parent, source) = self {
            Some(( file, parent, source ))
        } else {
            None
        }
    }

    pub fn am_i_tokenized_file(&self) -> bool {
        self.get_my_tokenized_file().is_some()
    }

    pub fn is_tokenized_file(i : &Item) -> bool {
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

impl<E : CtxTrait> BaseNode<Item, E> {
    pub fn is_empty_comment(&self) -> bool {
        match self.item() {
            Item::Comment(text) => text.is_empty(),
            _ => false
        }
    }

    pub fn from_number<X>(n : i64, ctx : X) -> Self
        where X : Into<E>
        {
            Self::from_item(Item::Number(n),ctx.into())
        }

    pub fn to_label(txt : &str, ctx : E) -> Self {
        Self::from_item(Item::Label(txt.to_string()), ctx)
    }
    pub fn to_local_label(txt : &str, ctx : E) -> Self {
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
            _ => None
        }
    }

    pub fn is_include_file(&self) -> bool {
        self.get_include_file().is_some()
    }
}

pub fn join_vec<I : Display>(v : &[I], sep : &str) -> String {
    let ret : Vec<_> = v.iter().map(|x| x.to_string()).collect();
    ret.join(sep)

}

impl<'a> Display for BaseNode<Item,Position> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Item::*;

        let item = self.item();

        let join_children = |sep| join_vec(&self.children, sep) ;

        let ret : String = match item {
            LocalAssignmentFromPc(name) | AssignmentFromPc(name) => {
                format!("{} equ {}", name, self.children[0])
            },
            Pc => "*".to_string(),

            Label(name) | LocalLabel(name) => name.clone(),

            Comment(comment) => comment.clone(),
            QuotedString(test) => format!("\"{}\"", test),
            Register(r) => r.to_string(),

            RegisterList(vec) => {
                join_vec(vec, ",")
            },

            LocalAssignment(name) |
                Assignment(name) => {
                    format!("{} equ {}", name, self.children[0])
                },

            Expr => {
                join_children("")
            }

            Include(file) => format!("include \"{}\"",file.to_string_lossy()),

            Number(n) => {
                n.to_string()
            }
            UnaryMinus => "-".to_string(),
            UnaryTerm => {
                join_children("")
            }

            Mul => "*".to_string(),
            Div => "/".to_string(),
            Add => "+".to_string(),
            Sub => "-".to_string(),
            And => "&".to_string(),
            Or => "|".to_string(),
            Xor => "^".to_string(),
            Org => {
                format!("org {}", self.children[0] )
            },

            BracketedExpr => {
                format!("({})", join_children(""))
            }

            TokenizedFile(file, _, _) => {
                let header = format!("; included file {}", file.to_string_lossy());
                let children : Vec<String> = self.children.iter().map(|n| format!("{}", &*n)).collect();
                format!("{}\n{}", header, children.join("\n"))
            }

            _ => format!("{:?} not implemented", item)
        };

        write!(f, "{}", ret)
    }
}



