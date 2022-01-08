use std::{fmt::{Debug, DebugMap, Display}, usize};

use nom_locate::LocatedSpan;
use nom::{InputTake, Offset};

use crate::commands::parse_command;


pub type Span<'a> = LocatedSpan<&'a str>;

pub fn to_range(input: Span, rest: Span) -> std::ops::Range<usize> {
    let start = input.location_offset() ;
    let end = rest.location_offset() ;
    start..end
}

pub fn matched_span<'a>(input: Span<'a>, rest: Span<'a>) -> Span<'a> {
    let r = to_range(input, rest);
    input.take(r.len())
}

#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub line : usize,
    pub col: usize,
    pub range: std::ops::Range<usize>,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}:{})", self.line, self.col)
    }
}

impl<'a> Default for Position {
    fn default() -> Self {
        todo!()
    }
}

impl <'a> From<Span<'a>> for Position {
    fn from(i : Span<'a>) -> Self {
        let start = i.location_offset();
        let range = start .. (start + i.len());
        Position::new(i.location_line() as usize, i.get_column() as usize, range)
    }
}

impl crate::node::CtxTrait for Position { }

impl Position {
    pub fn new(line : usize, col: usize, range: std::ops::Range<usize>) -> Self {
        Self {line,col, range}
    }
}


