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
    NotSure(String),
    Label(String),
    LocalLabel(String),
    Comment(String),
    Assignment(Box<Item>, Box<Item>),
    String(String),
    Op(String),
    OpenBracket,
    CloseBracket,
    OpenSqBracket,
    CloseSqBracket,
    Comma,
    Hash,
    Plus,
    PlusPlus,
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
    IndexedSimple(Box<Item>, Box<Item>),
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
pub enum OpCode<'a> {
    NoArg(&'a str),
    WithArg(&'a str),
    RegisterList(&'a str, Vec<&'a str>)
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
    Include(String),
    Generic(String, Option<String>),
    Org(Box<Item>),
    Fdb(Vec<Item>),
    Fill(Box<Item>,Box<Item>),
}

pub fn is_empty_comment<'a>(item : &'a Item) -> bool {
    if let Item::Comment(com) = item {
        com.is_empty()
    } else {
        false
    }
}

