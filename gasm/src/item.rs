use std::path::PathBuf;

use emu::cpu::RegEnum;
use nom::IResult;

use crate::fileloader::FileLoader;

pub type NodeResult<'a> = IResult<&'a str, Node>;

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
    Assignment(Box<Item>, Box<Item>),
    OpCodeWithArg(String, Box<Item>),
    Indexed(Box<Item>, Box<Item>),
    Immediate(Box<Item>),
    Indirect(Box<Item>),
    DirectPage(Box<Item>),
    Expr(Vec<Item>),
    Expr2,

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
    Command(Command),
    Eof,
    Register(RegEnum),
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
pub enum Command {
    Include(PathBuf),
    Generic(String, Option<String>),
    Org(Box<Item>),
    Fdb(Vec<Item>),
    Fill(Box<Item>,Box<Item>),
    Zmb(Box<Item>)
}

pub struct Node {
    item: Item,
    children: Vec<Box<Node>>
}

impl Node {
    pub fn new(item : Item, children: Vec<Box<Node>>) -> Self {
        Self {item, children}
    }

}

impl From<Item> for Node {
    fn from(item : Item) -> Node {
        Node::new(item, vec![])
    }
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


pub struct Parser {
    text : String,
    offset: usize,
}

fn get_offset(master: &str, text: &str) -> usize {
    text.as_ptr() as usize - master.as_ptr() as usize
}

impl Parser {
    pub fn parse<'a, P, E>(&'a mut self, mut p : P) -> IResult<&'a str, Item, E>
        where 
        P: nom::Parser<&'a str, Item, E>,
        E: nom::error::ParseError<&'a str>,
    {
            let input = &self.text[self.offset..];

            let (rest, matched) = p.parse(input)?;

            let offset = get_offset(input,rest);
            self.offset = offset;

            Ok((rest,  matched ))
    }
}


