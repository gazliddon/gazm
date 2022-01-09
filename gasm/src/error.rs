use std::fmt::Display;
use std::path::PathBuf;

use nom::{self, Offset};

use nom::AsBytes;
use crate::locate::{ Span, Position };

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    pub span: Span<'a>,
    pub message: Option<String>,
}

impl<'a> ParseError<'a> {
    pub fn message(&self) -> String {
        self.message.clone().unwrap_or("".to_string())
    }
}

pub fn error<'a>(err : &str, ctx: Span<'a>) -> nom::Err<ParseError<'a>> {
    nom::Err::Error(ParseError::new( err.to_string(), &ctx))
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
pub struct UserError {
    pub message: String,
    pub pos: Position,
    pub fragment: String,
    pub line: String,
    pub file: std::path::PathBuf,
}

impl UserError {
    pub fn from_parse_error(err : ParseError, file : &std::path::PathBuf) -> Self {
        let line = err.span.get_line_beginning();
        let line = String::from_utf8_lossy(line).to_string();
        Self {
            message: err.message(),
            pos: err.span.into(),
            fragment: err.span.to_string(),
            line, file: file.clone()
        }
    }
}


