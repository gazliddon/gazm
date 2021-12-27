use nom::Offset;
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str, &'a str>;

#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub start: usize,
    pub end: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self { start: 0, end: 0 }
    }
}

impl crate::node::CtxTrait for Position {
}

impl Position {
    pub fn new(start : Span, end : Span) -> Self {
        use nom_locate::position;
        use super::error::ParseError;

        let start = start.extra.offset(&start);
        let end = end.extra.offset(&end);

        Self {
            start,
            end
        }
    }
}


