
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
    NotSure2(TextItem<'a>),
    Arg(&'a str),
    Label(&'a str),
    LocalLabel(&'a str),
    Comment(&'a str),
    Assignment(Box<Item<'a>>, &'a str),
    String(&'a str),
    BinOp(&'a str),
    OpenBracket,
    CloseBracket,
    OpenSqBracket,
    CloseSqBracket,
    Comma,
    Hash,
    Plus,
    PlusPlus,
    Number(u64, &'a str),
    ArgList(Vec<Item<'a>>),
    OpCode(&'a str, Option<Box<Item<'a>>>),
    OpCodeWithArg(&'a str, &'a str),
    Command(Command<'a>),
    Eof,
}

enum OpCode<'a> {
    NoArg(&'a str),
    WithArg(&'a str),
    RegisterList(&'a str, Vec<&'a str>)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command<'a> {
    Include(&'a str),
    Generic(&'a str, Option<&'a str>)
}

pub fn is_empty_comment<'a>(item : &'a Item<'a>) -> bool {
    if let Item::Comment(com) = *item {
        com.is_empty()
    } else {
        false
    }
}

