use nom::Offset;
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str, &'a str>;

pub trait AsSpan<'a> {
    fn as_span(&'a self) -> Span<'a>;
}

impl<'a> AsSpan<'a> for str {
    fn as_span(&'a self) -> Span<'a> {
        Span::new_extra(self,self)
    }
}
impl<'a> AsSpan<'a> for String {
    fn as_span(&'a self) -> Span<'a> {
        Span::new_extra(self.as_str(),self.as_str())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Span2<'a> {
    span : Span<'a>
}

use std::ops::Deref;

impl<'a> Span2<'a> {

    pub fn new(_a: &'a str) -> Self {
        panic!()
    }
}
impl<'a> Deref for Span2<'a> {
    type Target = LocatedSpan<&'a str, &'a str>;

    fn deref(&self) -> &Self::Target {
        &self.span
    }
}

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

impl crate::node::CtxTrait for Position { }

impl Position {
    pub fn from_usize(pos :(usize, usize)) -> Self {
        let (start,end) = pos;
        Self {
            start,end
        }
    }

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


