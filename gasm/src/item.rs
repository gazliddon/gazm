use std::{path::PathBuf, slice::Iter};

use emu::cpu::RegEnum;
use nom::IResult;

use crate::fileloader::FileLoader;

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


pub struct NodeIt<'a, CTX : Default> {
    index : usize,
    node : &'a BaseNode<CTX>
}

impl<'a, CTX : Default> NodeIt<'a, CTX > {
    pub fn new(node : &'a BaseNode<CTX>) -> Self {
        Self { index: 0, node }
    }
}

impl<'a, CTX: Default> Iterator for NodeIt<'a, CTX> {
    type Item = &'a BaseNode<CTX>;

    fn next(&mut self) -> Option<&'a BaseNode<CTX>> {
        if let Some(ret) = self.node.children.get(self.index) {
            self.index = self.index + 1;
            Some(ret)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
}
#[derive(PartialEq, Clone)]
pub struct BaseNode<CTX : Default> {
    pub item: Item,
    pub children: Vec<Box<BaseNode<CTX>>>,
    pub ctx: CTX,
}

impl<CTX : Default> BaseNode<CTX> {

    pub fn iter(&self) -> NodeIt<CTX> {
        NodeIt::new(self)
    }

    pub fn from_item(item: Item) -> Self {
        Self::new(item, vec![], CTX::default())
    }

    pub fn new(item : Item, children: Vec<Box<Self>>, ctx : CTX) -> Self {
        Self {item, children, ctx
    }
    }
    pub fn with_items(self, _children : Vec<Item>) -> Self {
        panic!()
    }

    pub fn with_children(self, children : Vec<BaseNode<CTX>>) -> Self {
        let mut ret = self;
        ret.children = children.into_iter().map(Box::new).collect();
        ret
    }
    pub fn with_child(self, child : Self) -> Self {
        let mut ret = self;
        ret.children = vec![child.into()];
        ret
    }

    pub fn with_ctx(self, ctx : CTX) -> Self {
        let mut ret = self;
        ret.ctx = ctx;
        ret
    }

    pub fn is_empty_comment(&self) -> bool {
        match &self.item {
            Item::Comment(s) => s.is_empty(),
            _ => false
        }
    }

    pub fn from_number(num : i64 ) -> Self {
        Self::from_item(Item::Number(num))
    }
}

////////////////////////////////////////////////////////////////////////////////
impl<CTX: Default> std::fmt::Debug for BaseNode<CTX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.item)
    }
}

pub type Node = BaseNode<()>;

impl From<Node> for Item {
    fn from(node : Node) -> Self {
        node.item
    }
}

impl From<Item> for Node {
    fn from(item : Item) -> Node {
        Node::from_item(item)
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


    pub fn number(n : i64) -> Self {
        Item::Number(n)
    }

}

impl Into<Box<Item>> for Node {
    fn into(self) -> Box<Item> {
        Box::new(self.item)
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
    pub fn parse<'a, P, E>(&'a mut self, mut p : P) -> IResult<&'a str, Node, E>
        where 
        P: nom::Parser<&'a str, Node, E>,
        E: nom::error::ParseError<&'a str>,
        {
            let input = &self.text[self.offset..];

            let (rest, matched) = p.parse(input)?;

            let offset = get_offset(input,rest);
            self.offset = offset;

            Ok((rest,  matched ))
        }
}


