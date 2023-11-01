#![deny(unused_imports)]
use std::collections::HashMap;

use super::basetoken::Token as BaseToken;
use super::parsetext::*;
use emu6809::cpu::RegEnum;

use emu6809::isa::Dbase;

use logos::{Lexer, Logos};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum NumberKind {
    Hex,
    Dec,
    Bin,
    Char,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, EnumIter, Hash)]
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
    Rmd,
    Rzb,
    Org,
    Include,
    Exec,
    Require,
    Import,
    Struct,
    Macro,
    Equ,
}

impl From<CommandKind> for TokenKind {
    fn from(value: CommandKind) -> Self {
        TokenKind::Identifier(IdentifierKind::Command(value))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IdentifierKind {
    Command(CommandKind),
    Opcode,
    Label,
    Register(RegEnum),
}

impl From<RegEnum> for TokenKind {
    fn from(value: RegEnum) -> Self {
        TokenKind::Identifier(IdentifierKind::Register(value))
    }
}

impl From<IdentifierKind> for TokenKind {
    fn from(value: IdentifierKind) -> Self {
        TokenKind::Identifier(value)
    }
}

lazy_static::lazy_static! {
    static ref COMS : HashMap<String, CommandKind> = {

    let hash: HashMap<String, CommandKind> = CommandKind::iter()
        .map(|com| (format!("{:?}", com).to_lowercase(), com))
        .collect();

    hash
    };

    static ref DBASE_6809 : Dbase = Dbase::new();
}

trait CpuLexer {
    fn identifier(&self, text: &str) -> Option<IdentifierKind>;
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Default)]
struct Cpu6809Lexer {}

impl Cpu6809Lexer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn as_register(text: &str) -> Option<IdentifierKind> {
        use RegEnum::*;
        match text {
            "a" => Some(A.into()),
            "b" => Some(B.into()),
            "d" => Some(D.into()),
            "x" => Some(X.into()),
            "y" => Some(Y.into()),
            "u" => Some(U.into()),
            "s" => Some(S.into()),
            "dp" => Some(DP.into()),
            "cc" => Some(CC.into()),
            "pc" => Some(PC.into()),
            _ => None,
        }
    }
}

impl CpuLexer for Cpu6809Lexer {
    fn identifier(&self, text: &str) -> Option<IdentifierKind> {
        use IdentifierKind::*;
        if DBASE_6809.get_opcode(text).is_some() {
            Some(Opcode)
        } else {
            Self::as_register(text)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

lazy_static::lazy_static! {
    static ref PRE_HEX : regex::Regex = regex::Regex::new(r"(0[xX]|\$)(.*)").unwrap();
    static ref PRE_BIN : regex::Regex = regex::Regex::new(r"(0[bB]|%)(.*)").unwrap();
}

impl From<RegEnum> for IdentifierKind {
    fn from(value: RegEnum) -> Self {
        IdentifierKind::Register(value)
    }
}

fn identifier(lex: &mut Lexer<TokenKind>) -> Option<IdentifierKind> {
    use IdentifierKind::*;
    let cpu_lex = Cpu6809Lexer::new();
    let text = lex.slice().to_lowercase();

    if let Some(c) = COMS.get(&text) {
        Some(Command(*c))
    } else if let Some(x) = cpu_lex.identifier(&text) {
        Some(x)
    } else {
        Some(IdentifierKind::Label)
    }
}

fn get_num(txt: &str, re: &regex::Regex, radix: usize) -> i64 {
    let caps = re.captures(txt).unwrap();
    let num_str = caps.get(2).unwrap().as_str().replace('_', "");
    i64::from_str_radix(&num_str, radix as u32).unwrap()
}

fn from_bin(lex: &mut Lexer<TokenKind>) -> Option<(i64, NumberKind)> {
    let num = get_num(lex.slice(), &PRE_BIN, 2);
    Some((num, NumberKind::Bin))
}

fn from_hex(lex: &mut Lexer<TokenKind>) -> Option<(i64, NumberKind)> {
    let num = get_num(lex.slice(), &PRE_HEX, 16);
    Some((num, NumberKind::Hex))
}
fn from_char(lex: &mut Lexer<TokenKind>) -> Option<(i64, NumberKind)> {
    let num = get_num(lex.slice(), &PRE_HEX, 16);
    Some((num, NumberKind::Hex))
}

fn from_dec(lex: &mut Lexer<TokenKind>) -> Option<(i64, NumberKind)> {
    let num: i64 = lex.slice().replace('_', "").parse().unwrap();
    Some((num, NumberKind::Dec))
}

#[derive(Default)]
pub struct State {}

#[derive(Logos, Copy, Clone, Debug, PartialEq, Eq)]
#[logos(extras = State)]
#[logos(skip r"[ \t\f\n]+")]
#[logos(subpattern id_al = r"[a-zA-Z_.]")]
#[logos(subpattern id_alnum = r"(?&id_al)|[0-9]")]
#[logos(subpattern id = r"(?&id_al)+(?&id_alnum)*")]
#[logos(subpattern pre_hex = r"(0[xX]|\$)")]
#[logos(subpattern pre_bin = r"(0[bB]|%)")]
pub enum TokenKind {
    Error,

    // #[regex(r"\[\[[^\]]*\]\]", priority=10)]
    #[regex(r"```[^`]*```", priority = 10)]
    BigDocText,

    #[regex("(?&id)", identifier)]
    Identifier(IdentifierKind),

    #[regex(r"[0-9][0-9_]*", from_dec)]
    #[regex(r"(?&pre_hex)[0-9a-fA-F][0-9a-fA-F_]*", from_hex)]
    #[regex(r"(?&pre_bin)[0-1][0-1_]*", from_bin)]
    Number((i64, NumberKind)),

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

    #[regex(r"'.'", from_char)]
    Char((i64, NumberKind)),

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)]
    QuotedString,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

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

pub type Token<'a> = BaseToken<ParseText<'a>>;

pub fn tokenize_6809<'a>(token: &Token<'a>) -> Token<'a> {
    *token
}

pub fn to_tokens_kinds(
    source_file: &grl_sources::SourceFile,
) -> Vec<(TokenKind, std::ops::Range<usize>)> {
    TokenKind::lexer(&source_file.source.source)
        .spanned()
        .map(|(tok_res, pos)| match tok_res {
            Ok(kind) => (kind, pos),
            Err(_) => (TokenKind::Error, pos),
        })
        .collect()
}

fn to_tokens(source_file: &grl_sources::SourceFile) -> Vec<Token> {
    let ret = to_tokens_kinds(source_file);

    ret.into_iter()
        .map(|(tk, r)| {
            let pt = ParseText::new(source_file, r);
            let t: Token = Token::new(tk, pt);
            t
        })
        .collect()
}

pub fn to_tokens_no_comment(source_file: &grl_sources::SourceFile) -> Vec<Token> { 
    use TokenKind::*;
    let not_comment = |k: &TokenKind| k != &DocComment && k != &Comment;
    let tokens = to_tokens_filter(source_file, not_comment);
    tokens
}

pub fn to_tokens_filter<P>(source_file: &grl_sources::SourceFile, predicate: P) -> Vec<Token>
where
    P: Fn(&TokenKind) -> bool,
{
    let ret = to_tokens_kinds(source_file);

    ret.into_iter()
        .filter(|(tk, _)| predicate(tk))
        .map(|(tk, r)| {
            let pt = ParseText::new(source_file, r);
            let t: Token = Token::new(tk,  pt);
            t
        })
        .collect()
}
