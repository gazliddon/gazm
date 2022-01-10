use std::fmt::Display;
use std::path::PathBuf;

use nom::{self, Offset};

use nom::AsBytes;
use crate::locate::{ Span, Position };
use crate::ast::AstNodeRef;


#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    pub span: Span<'a>,
    pub message: Option<String>,
}

impl<'a> ParseError<'a> {
    pub fn message(&self) -> String {
        self.message.clone().unwrap_or_else(||"".to_string())
    }
}

pub fn error<'a>(err : &str, ctx: Span<'a>) -> nom::Err<ParseError<'a>> {
    nom::Err::Error(ParseError::new( err.to_string(), &ctx))
}
pub fn user_error(_err : &str, _ctx: Span) -> UserError {
    panic!()
}

pub fn failure<'a>(err : &str, ctx: Span<'a>) -> nom::Err<ParseError<'a>> {
    nom::Err::Failure(ParseError::new( err.to_string(), &ctx))
}

pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError<'a>>;

impl<'a> ParseError<'a> {
    pub fn new(message: String, span: &Span<'a>) -> ParseError<'a> {
        Self { span: *span, message: Some(message), }
    }

    // pub fn span(&self) -> &Span { &self.span }

    pub fn line(&self) -> usize { 
        panic!()
    }

    pub fn offset(&self) -> usize {
        panic!()
    }

    pub fn fragment(&self) -> &'a str {
        &self.span
    } 

    // pub fn from_text(message : &str) -> Self {
    //     Self {message: Some(message.to_string()),
    //     pos : Default::default() }
    // }
}

impl<'a> From<nom::Err<ParseError<'a>>> for ParseError<'a> {
    fn from(i: nom::Err<ParseError<'a>>) -> Self {
        match i {
            nom::Err::Incomplete(_) => panic!(),
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
        }
    }
}

// That's what makes it nom-compatible.
impl<'a> nom::error::ParseError<Span<'a>> for ParseError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {:?}", kind), &input)
    }

    fn append(_input: Span, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("unexpected character '{}'", c), &input)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AstError {
    pub pos : Position,
    pub message: Option<String>,
}

impl AstError {
    pub fn new(msg: &str, pos : &Position) -> Self {
        Self {
            pos : pos.clone(),
            message :  Some(msg.to_string()),
        }
    }

    pub fn from_node(msg: &str, n : AstNodeRef) -> Self {
        Self {
            pos : n.value().pos.clone(),
            message :  Some(msg.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct UserError {
    pub message: String,
    pub pos: Position,
    pub fragment: String,
    pub line: String,
    pub file: std::path::PathBuf,
}

impl UserError {
    pub fn from_ast_error(_err : AstError, _file : &std::path::Path) -> Self {
        panic!()
    }

    pub fn from_parse_error(err : ParseError, file : &std::path::Path) -> Self {
        let line = err.span.get_line_beginning();
        let line = String::from_utf8_lossy(line).to_string();
        Self {
            message: err.message(),
            pos: err.span.into(),
            fragment: err.span.to_string(),
            line, file: file.to_path_buf()
        }
    }
}


