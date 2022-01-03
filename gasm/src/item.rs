use std::fmt::Display;
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

impl<'a> CtxTrait for Span<'a> { }

pub type Node = BaseNode<Item, Position>;

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Assignment,
    AssignmentFromPc(String),
    LocalAssignmentFromPc(String),
    Expr,
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
    // Op(String),
    Number(i64),
    OpCode(String, Instruction),
    Operand(AddrModeEnum),
    Register(RegEnum),
    PreDecrement(RegEnum),
    PreIncrement(RegEnum),
    DoublePreDecrement(RegEnum),
    DoublePreIncrement(RegEnum),
    PostDecrement(RegEnum),
    PostIncrement(RegEnum),
    DoublePostDecrement(RegEnum),
    DoublePostIncrement(RegEnum),

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

impl Item {
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
            Some(( &file, &parent, &source ))
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
            Some(&name)
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

pub fn join_vec<I : Display>(v : &Vec<I>, sep : &str) -> String {
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

            Assignment => {
                format!("{} equ {}", self.children[0], self.children[1])
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



