#![deny(unused_imports)]

use super::{basetoken::Token as BaseToken, ParseText};
use logos::{Lexer, Logos};
use std::collections::HashMap;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

pub type Token<'a> = BaseToken<ParseText<'a>>;

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

lazy_static::lazy_static! {
    pub static ref COMS : HashMap<String, CommandKind> = {

    let hash: HashMap<String, CommandKind> = CommandKind::iter()
        .map(|com| (format!("{:?}", com).to_lowercase(), com))
        .collect();

    hash
    };
}

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
lazy_static::lazy_static! {
    static ref PRE_HEX : regex::Regex = regex::Regex::new(r"(0[xX]|\$)(.*)").unwrap();
    static ref PRE_BIN : regex::Regex = regex::Regex::new(r"(0[bB]|%)(.*)").unwrap();
}

fn get_num(txt: &str, re: &regex::Regex, radix: usize) -> Option<i64> {
    re.captures(txt).map(|caps| {
        let num_str = caps.get(2).unwrap().as_str().replace('_', "");
        i64::from_str_radix(&num_str, radix as u32).unwrap()
    })
}

fn from_bin(lex: &mut Lexer<TokenKind>) -> Option<(i64, NumberKind)> {
    get_num(lex.slice(), &PRE_BIN, 2).map(|num| (num, NumberKind::Bin))
}

fn from_hex(lex: &mut Lexer<TokenKind>) -> Option<(i64, NumberKind)> {
    get_num(lex.slice(), &PRE_HEX, 16).map(|num| (num, NumberKind::Hex))
}

fn from_char(lex: &mut Lexer<TokenKind>) -> Option<(i64, NumberKind)> {
    lex.slice()
        .as_bytes()
        .get(1)
        .map(|c| (*c as i64, NumberKind::Char))
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

    OpCode,
    Command(CommandKind),
    Label,

    // #[regex(r"\[\[[^\]]*\]\]", priority=10)]
    #[regex(r"```[^`]*```", priority = 10)]
    BigDocText,

    #[regex("(?&id)")]
    TempIdentifier,

    #[regex(r"[0-9][0-9_]*", from_dec)]
    #[regex(r"(?&pre_hex)[0-9a-fA-F][0-9a-fA-F_]*", from_hex)]
    #[regex(r"(?&pre_bin)[0-1][0-1_]*", from_bin)]
    #[regex(r"'.'", from_char)]
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

    #[regex(r"::(?&id)(::(?&id))+")]
    FqnIdentifier,

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


pub fn map_token(
    kind: TokenKind,
    pos: std::ops::Range<usize>,
    source_file: &grl_sources::SourceFile,
) -> (TokenKind, std::ops::Range<usize>)
{
    let kind = match kind {
        TokenKind::TempIdentifier => {
            let text = &source_file.get_text().source[pos.clone()].to_lowercase();

            if let Some(c) = COMS.get(text) {
                TokenKind::Command(*c)
            } else {
                todo!()
                // C::lex_identifier(text.as_str())
            }
        }

        _ => kind,
    };
    (kind, pos)
}

pub fn to_tokens_no_comment(source_file: &grl_sources::SourceFile) -> Vec<Token>
{
    use TokenKind::*;
    let not_comment = |k: &TokenKind| k != &DocComment && k != &Comment;
    let tokens = to_tokens_filter(source_file, not_comment);
    tokens
}

fn to_tokens_kinds(
    source_file: &grl_sources::SourceFile,
) -> Vec<(TokenKind, std::ops::Range<usize>)>
{
    TokenKind::lexer(&source_file.get_text().source)
        .spanned()
        .map(|(tok_res, pos)| match tok_res {
            Ok(kind) => map_token(kind, pos, source_file),
            Err(_) => (TokenKind::Error, pos),
        })
        .collect()
}

fn to_tokens_filter< P>(source_file: &grl_sources::SourceFile, predicate: P) -> Vec<Token>
where
    P: Fn(&TokenKind) -> bool,
{
    let ret = to_tokens_kinds(source_file);

    ret.into_iter()
        .filter(|(tk, _)| predicate(tk))
        .map(|(tk, r)| {
            let pt = ParseText::new(source_file, r);
            let t: Token = Token::new(tk, pt);
            t
        })
        .collect()
}
