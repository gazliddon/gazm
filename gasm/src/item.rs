use std::{path::PathBuf, slice::Iter};

use emu::cpu::RegEnum;
use nom::IResult;

use crate::fileloader::FileLoader;
use crate::node::{ BaseNode};

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    File(PathBuf),
    Assignment,
    OpCodeWithArg(String),
    Indexed,
    Immediate,
    Indirect,
    DirectPage,
    Expr,
    Pc,

    UnaryTerm,

    RegisterList(Vec<RegEnum>),
    Label(String),
    LocalLabel(String),
    Comment(String),
    QuotedString(String),
    Op(String),
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
    Generic(String, Option<String>),

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
pub type Node = BaseNode<Item>;

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

impl Into<Box<Item>> for Node {
    fn into(self) -> Box<Item> {
        Box::new(self.item().clone())
    }
}

// impl From<BaseNode<Item>> for Item {
//     fn from(node : BaseNode<Item>) -> Self {
//         node.item
//     }
// }

impl BaseNode<Item> {
    pub fn is_empty_comment(&self) -> bool {
        match self.item() {
            Item::Comment(text) => text.is_empty(),
            _ => false
        }
    }
    pub fn from_number(n : i64) -> Self {
        Self::from_item(Item::Number(n))
    }
}

fn get_offset(master: &str, text: &str) -> usize {
    text.as_ptr() as usize - master.as_ptr() as usize
}
