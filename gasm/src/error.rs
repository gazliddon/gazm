use std::path::PathBuf;

use nom;

use nom::AsBytes;
use crate::locate::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    span: Span<'a>,
    message: Option<String>,
}

pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError<'a>>;
// the trait `From<nom::Err<E>>` is not implemented for `nom::Err<nom::error::Error<LocatedSpan<&str>>>`

impl<'a> ParseError<'a> {
    pub fn new(message: String, span: Span<'a>) -> Self {
        Self { span, message: Some(message) }
    }

    pub fn span(&self) -> &Span { &self.span }

    pub fn line(&self) -> u32 { self.span().location_line() }

    pub fn offset(&self) -> usize { self.span().location_offset() }
}

// That's what makes it nom-compatible.
impl<'a> nom::error::ParseError<Span<'a>> for ParseError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {:?}", kind), input)
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("unexpected character '{}'", c), input)
    }
}
