use std::{fmt::{Debug, DebugMap, Display}, usize};

use nom_locate::LocatedSpan;
use nom::{InputTake, Offset};

use crate::commands::parse_command;
use romloader::{Position, AsmSource};

pub type Span<'a> = LocatedSpan<&'a str, AsmSource>;

pub fn to_range(input: Span, rest: Span) -> std::ops::Range<usize> {
    let start = input.location_offset() ;
    let end = rest.location_offset() ;
    start..end
}

pub fn matched_span<'a>(input: Span<'a>, rest: Span<'a>) -> Span<'a> {
    let r = to_range(input, rest);
    input.take(r.len())
}

pub fn to_pos<'a>(i : Span<'a>) -> Position {
        let start = i.location_offset();
        let range = start .. (start + i.len());
        Position::new(i.location_line() as usize, i.get_column() as usize, range, i.extra)
}

impl crate::node::CtxTrait for Position { }

