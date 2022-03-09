use nom_locate::LocatedSpan;
use nom::InputTake;
use romloader::sources::{Position, AsmSource};

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

pub fn span_to_pos(i : Span) -> Position {
        let start = i.location_offset();
        let range = start .. (start + i.len());
        Position::new(i.location_line() as usize, i.get_column() as usize, range, i.extra)
}

impl crate::node::CtxTrait for Position { }

