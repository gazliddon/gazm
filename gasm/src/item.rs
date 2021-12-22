use std::path::PathBuf;

use emu::cpu::RegEnum;

#[derive(Debug, PartialEq, Clone)]
pub struct TextItem<'a> {
    pub offset: usize,
    pub text: &'a str,
}

impl<'a> TextItem<'a> {
    pub fn from_slice(master: &'a str, text: &'a str) -> Self {
        let offset = text.as_ptr() as usize - master.as_ptr() as usize;
        TextItem { text, offset }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Label(String),
    LocalLabel(String),
    Comment(String),
    Assignment(Box<Item>, Box<Item>),
    QuotedString(String),
    Op(String),
    OpenBracket,
    CloseBracket,
    Number(i64),
    ArgList(Vec<Item>),
    OpCode(String),
    OpCodeWithArg(String, Box<Item>),
    Command(Command),
    Eof,
    Register(RegEnum),
    RegisterList(Vec<RegEnum>),
    Expr(Vec<Item>),
    Immediate(Box<Item>),
    Indirect(Box<Item>),
    DirectPage(Box<Item>),
    Indexed(Box<Item>, Box<Item>),
    PreDecrement(RegEnum),
    PreIncrement(RegEnum),
    DoublePreDecrement(RegEnum),
    DoublePreIncrement(RegEnum),
    PostDecrement(RegEnum),
    PostIncrement(RegEnum),
    DoublePostDecrement(RegEnum),
    DoublePostIncrement(RegEnum),
}


#[derive(Debug, PartialEq, Clone)]
pub struct Location<'a> {
    line : usize,
    column : usize,
    text : &'a str,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    item : Item,
    location: Location<'a>,
    children: Vec<Token<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Include(PathBuf),
    Generic(String, Option<String>),
    Org(Box<Item>),
    Fdb(Vec<Item>),
    Fill(Box<Item>,Box<Item>),
    Zmb(Box<Item>)
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

    pub fn zero_expr() -> Self {
        Self::Expr(vec![Self::zero()])
    }

    pub fn number(n : i64) -> Self {
        Item::Number(n)
    }
}


