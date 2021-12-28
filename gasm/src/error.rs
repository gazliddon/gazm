use std::path::PathBuf;

use nom::{self, Offset};

use nom::AsBytes;
use crate::locate::{ Span, Position };

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    pub pos: Position,
    pub message: Option<String>,
}

pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError>;

impl ParseError {
    pub fn new(message: String, span: Span) -> Self {
        let start = span.extra.offset(span.fragment());
        let end = start + span.len();
        let pos = Position::from_usize((start, end));
        Self { pos, message: Some(message) }
    }

    // pub fn span(&self) -> &Span { &self.span }

    pub fn line(&self) -> usize { 
        panic!()
    }

    pub fn offset(&self) -> usize {
        panic!()
    }

    pub fn from_text(message : &str) -> Self {
        Self {message: Some(message.to_string()),
        pos : Default::default() }
    }
}

impl From<nom::Err<ParseError>> for ParseError {
    fn from(i: nom::Err<ParseError>) -> Self {
        match i {
            nom::Err::Incomplete(_) => panic!(),
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
        }
    }
}

// That's what makes it nom-compatible.
impl<'a> nom::error::ParseError<Span<'a>> for ParseError {
    fn from_error_kind(input: Span, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {:?}", kind), input)
    }

    fn append(_input: Span, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span, c: char) -> Self {
        Self::new(format!("unexpected character '{}'", c), input)
    }
}


