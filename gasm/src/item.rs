use std::{path::PathBuf, slice::Iter, collections::HashSet};

use emu::cpu::RegEnum;
use nom::IResult;

use crate::fileloader::FileLoader;
use crate::node::{BaseNode, CtxTrait};
use crate::ctx::Ctx;
use crate::locate::Span;

use crate::locate::Position;

pub type Node = BaseNode<Item, Position>;

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    File(PathBuf),
    Assignment,
    Expr,
    Pc,

    Indexed,
    Immediate,
    Indirect,
    DirectPage,

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
    OpCode(String),
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
    pub fn from_number(n : i64) -> Self {
        Self::from_item(Item::Number(n))
    }

    pub fn to_label(txt : &str) -> Self {
        Self::from_item(Item::Label(txt.to_string()))
    }
    pub fn to_local_lable(txt : &str) -> Self {
        Self::from_item(Item::LocalLabel(txt.to_string()))
    }

    pub fn get_label_name(&self) -> Option<&String> {
        if let Item::Label(name) = self.item() {
            Some(&name)
        } else {
            None
        }
    }
}

impl BaseNode<Item, Position> {
    pub fn with_pos(self, start : Span, end : Span) -> Self {
        use super::locate::Position;
        let ctx = Position::new(start, end);
        self.with_ctx(ctx)
    }

    pub fn with_upos(self, start: usize, end: usize) -> Self {
        use super::locate::Position;
        let ctx = Position::from_usize((start,end));
        self.with_ctx(ctx)
    }
}

fn get_offset(master: &str, text: &str) -> usize {
    text.as_ptr() as usize - master.as_ptr() as usize
}


