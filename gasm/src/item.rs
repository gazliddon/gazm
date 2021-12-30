use std::path::Path;
use std::{path::PathBuf, slice::Iter, collections::HashSet};

use emu::cpu::RegEnum;
use emu::isa::{AddrModeEnum, Instruction, InstructionInfo};
use nom::{IResult, Offset};

use crate::fileloader::FileLoader;
use crate::node::{BaseNode, CtxTrait};
use crate::ctx::Ctx;
use crate::locate::Span;

use crate::locate::Position;

impl<'a> CtxTrait for Span<'a> { }

pub type Node = BaseNode<Item, Position>;

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Block,
    Assignment,
    Expr,
    Pc,

    UnaryTerm,

    RegisterList(Vec<RegEnum>),
    RegisterSet(HashSet<RegEnum>),
    Label(String),
    LocalLabel(String),
    Comment(String),
    QuotedString(String),
    // Op(String),
    OpenBracket,
    CloseBracket,
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

    TokenizedFile(PathBuf, PathBuf),

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
    pub fn to_local_lable(txt : &str, ctx : E) -> Self {
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

impl<'a> BaseNode<Item, Span<'a>> {
}



