use nom::InputTake;
use nom_locate::LocatedSpan;
use emu::utils::sources::{AsmSource, Position};

use crate::ctx::Opts;


// TODO: Replace ASM source with this?
#[derive(Clone, Debug, Copy)]
pub struct ParseCtx<'a> {
    opts: &'a Opts,
    src: AsmSource,
}

pub type Span<'a> = LocatedSpan<&'a str, AsmSource>;

pub fn to_range(input: Span, rest: Span) -> std::ops::Range<usize> {
    let start = input.location_offset();
    let end = rest.location_offset();
    start..end
}

pub fn matched_span<'a>(input: Span<'a>, rest: Span) -> Span<'a> {
    let r = to_range(input, rest);
    input.take(r.len())
}

pub fn span_to_pos(i: Span) -> Position {
    let start = i.location_offset();
    let range = start..(start + i.len());
    Position::new(
        i.location_line() as usize - 1 ,
        i.get_column() - 1,
        range,
        i.extra,
    )
}

impl crate::node::CtxTrait for Position {}
