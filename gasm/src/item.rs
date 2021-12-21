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
pub enum Item<'a> {
    NotSure(&'a str),
    Label(&'a str),
    LocalLabel(&'a str),
    Comment(&'a str),
    Assignment(Box<Item<'a>>, Box<Item<'a>>),
    String(&'a str),
    BinOp(&'a str, Box<Token<'a>>, Box<Token<'a>> ),
    Op(&'a str),
    OpenBracket,
    CloseBracket,
    OpenSqBracket,
    CloseSqBracket,
    Comma,
    Hash,
    Plus,
    PlusPlus,
    Number(i64),
    ArgList(Vec<Item<'a>>),
    OpCode(&'a str),
    OpCodeWithArg(&'a str, Box<Item<'a>>),
    Command(Command<'a>),
    Eof,
    Register(RegEnum),
    RegisterList(Vec<RegEnum>),
    Expr(Vec<Item<'a>>),
    Immediate(Box<Item<'a>>),
    Indirect(Box<Item<'a>>),
    DirectPage(Box<Item<'a>>),
    IndexedSimple(Box<Item<'a>>, Box<Item<'a>>),
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
    item : Item<'a>,
    location: Location<'a>,
    children: Vec<Token<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command<'a> {
    Include(&'a str),
    Generic(&'a str, Option<&'a str>),
    Org(Box<Item<'a>>),
    Fdb(Vec<Item<'a>>),
    Fill(Box<Item<'a>>,Box<Item<'a>>),
}

pub fn is_empty_comment<'a>(item : &'a Item<'a>) -> bool {
    if let Item::Comment(com) = *item {
        com.is_empty()
    } else {
        false
    }
}

