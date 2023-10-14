use std::collections::HashMap;

use emu6809::isa::Dbase;
use logos::{Lexer, Logos};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum NumberKind {
    Hex,
    Dec,
    Bin,
    Char
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, EnumIter,Hash)]
pub enum CommandKind {
    Scope,
    GrabMem,
    Put,
    IncBin,
    IncBinRef,
    WriteBin,
    SetDp,
    Bsz,
    Fill,
    Fdb,
    Fcc,
    Fcb,
    Zmb,
    Zmd,
    Rmb,
    Org,
    Include,
    Exec,
    Require,
    Import,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum IdentifierKind {
    Command(CommandKind),
    Opcode,
    Label,
}

lazy_static::lazy_static! {
    static ref COMS : HashMap<String, CommandKind> = {

    let hash: HashMap<String, CommandKind> = CommandKind::iter()
        .map(|com| (format!("{:?}", com).to_lowercase(), com))
        .collect();

    hash
    };

    static ref DBASE : Dbase = Dbase::new();

    static ref PRE_HEX : regex::Regex = regex::Regex::new(r"(0[xX]|\$)(.*)").unwrap();
    static ref PRE_BIN : regex::Regex = regex::Regex::new(r"(0[bB]|%)(.*)").unwrap();
}

fn identifier(lex: &mut Lexer<TokenKind>) -> Option<IdentifierKind> {
    let lc_com = lex.slice().to_lowercase();

    if let Some(c) = COMS.get(&lc_com) {
        Some(IdentifierKind::Command(c.clone()))
    } else if let Some(..) = DBASE.get_opcode(&lc_com) {
        Some(IdentifierKind::Opcode)
    } else {
        Some(IdentifierKind::Label)
    }
}

fn get_num(txt: &str, re: &regex::Regex, radix : usize) -> i64 {
    let caps = re.captures(txt).unwrap();
    let num_str = caps.get(2).unwrap().as_str().replace("_", "");
    i64::from_str_radix(&num_str, radix as u32).unwrap()
}

fn from_bin(lex: &mut Lexer<TokenKind>) -> Option<( i64,NumberKind )> {
    let num = get_num(lex.slice(), &PRE_BIN, 2);
    Some(( num, NumberKind::Bin ))
}

fn from_hex(lex: &mut Lexer<TokenKind>) -> Option<( i64,NumberKind )> {
    let num = get_num(lex.slice(), &PRE_HEX, 16);
    Some(( num,NumberKind::Hex))
}
fn from_char(lex: &mut Lexer<TokenKind>) -> Option<( i64,NumberKind )> {
    let num = get_num(lex.slice(), &PRE_HEX, 16);
    Some(( num,NumberKind::Hex))
}

fn from_dec(lex: &mut Lexer<TokenKind>) -> Option<( i64,NumberKind )> {
    let num :i64=  lex.slice().replace("_", "").parse().unwrap();
    Some(( num, NumberKind::Dec ))
}


// #[regex(r"([a-zA-Z-_]+[a-zA-Z0-9-_]*)(::[a-zA-Z-_]+[a-zA-Z0-9-_]*)+")]
#[derive(Logos, Copy, Clone, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\f\n]+")]
#[logos(subpattern id_al = r"[a-zA-Z_.]")]
#[logos(subpattern id_alnum = r"(?&id_al)|[0-9]")]
#[logos(subpattern id = r"(?&id_al)+(?&id_alnum)*")]
#[logos(subpattern pre_hex = r"(0[xX]|\$)")]
#[logos(subpattern pre_bin = r"(0[bB]|%)")]
pub enum TokenKind {
    Error,

    // #[regex(r"\[\[[^\]]*\]\]", priority=10)]
    #[regex(r"```[^`]*```", priority=10)]
    BigDocText,

    #[regex("(?&id)", identifier)]
    Identifier(IdentifierKind),

    #[regex(r"[0-9][0-9_]*",from_dec)]
    #[regex(r"(?&pre_hex)[0-9a-fA-F][0-9a-fA-F_]*", from_hex)]
    #[regex(r"(?&pre_bin)[0-1][0-1_]*",from_bin)]
    Number(( i64,NumberKind )),

    #[token("[")]
    OpenSquareBracket,

    #[token("]")]
    CloseSquareBracket,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("(")]
    OpenBracket,

    #[token(")")]
    CloseBracket,

    #[token("*")]
    Star,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("/")]
    Slash,

    // #[token("\\")]
    // BackSlash,

    #[regex(r";;;.*\n")]
    DocComment,

    #[regex(r"(;|//).*\n")]
    Comment,

    #[token("&")]
    Ampersand,

    #[regex(r"(?&id)(::(?&id))+")]
    FqnIdentifier,

    #[regex(r"'.'",from_char)]
    Char((i64, NumberKind )),

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)]
    QuotedString,

    #[token(",")]
    Comma,

    #[token(">>")]
    DoubleGreaterThan,

    #[token("<<")]
    DoubleLessThan,

    #[token(">")]
    GreaterThan,

    #[token("<")]
    LessThan,

    #[token("|")]
    Bar,

    #[token("^")]
    Caret,

    #[token("#")]
    Hash,

    #[token("!")]
    Pling,

    #[token("@")]
    At,
}

impl TokenKind {
    pub fn is_comment(&self) -> bool {
        self == &TokenKind::Comment
    }
}

impl From<std::ops::Range<usize>> for TextSpan {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self {
            start: value.start,
            len: value.len(),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Default)]
pub struct TextSpan {
    pub start: usize,
    pub len: usize,
}

impl TextSpan {
    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.start..self.start + self.len
    }
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParseText<'a> {
    pub base: &'a str,
    pub start: usize,
    pub len: usize,
}

impl<'a> ParseText<'a> {
    pub fn new(base: &'a str, range: std::ops::Range<usize>) -> Self {
        Self {
            base,
            start: range.start,
            len: range.len(),
        }
    }
}
impl<'a> ParseText<'a> {
    pub fn get_text(&self) -> &str {
        &self.base[self.as_range()]
    }

    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.start..self.start + self.len
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Token<X: Clone> {
    pub kind: TokenKind,
    pub location: TextSpan,
    pub extra: X,
}

impl<X: Clone> Token<X> {
    pub fn new(kind: TokenKind, location: TextSpan, extra: X) -> Self {
        Self {
            kind,
            location,
            extra,
        }
    }
}

impl<X: Clone> Token<X> {
    pub fn text_span(a: &[Self]) -> std::ops::Range<usize> {
        let start = a.first().unwrap().location.start;
        let end = a.last().unwrap().location.len + start;
        start..end
    }
}

pub fn to_tokens_kinds(source_file: &str) -> Vec<(TokenKind, std::ops::Range<usize>)> {
    TokenKind::lexer(source_file)
        .spanned()
        .map(|(tok_res, pos)| match tok_res {
            Ok(kind) => (kind, pos),
            Err(_) => (TokenKind::Error, pos),
        })
        .collect()
}

pub fn to_tokens(source_file: &str) -> Vec<Token<ParseText>> {
    let ret = to_tokens_kinds(source_file);

    ret.into_iter().map(|(tk,r)|  {
        let pt = ParseText::new(source_file,r.clone());
        let t : Token<ParseText> = Token::new(tk,r.into(),pt);
        t
    }).collect()
}


