use super::TokenKind;

#[derive(Clone, Debug, Copy, PartialEq, Default)]

pub struct TextSpan {
    pub start: usize,
    pub len: usize,
}

impl TextSpan {
    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.start..self.start + self.len
    }
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }
}

impl From<std::ops::Range<usize>> for TextSpan {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self {
            start: value.start,
            len: value.len(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Token<X: Clone> {
    pub kind: TokenKind,
    pub location: TextSpan,
    pub extra: X,
}

impl<X: Clone> Token<X> {
    pub fn new(kind: TokenKind, location: TextSpan, extra: X) -> Self {
        Self {
            kind,
            location,
            extra,
        }
    }
}

impl<X: Clone> Token<X> {
    pub fn text_span(a: &[Self]) -> std::ops::Range<usize> {
        let start = a.first().unwrap().location.start;
        let end = a.last().unwrap().location.len + start;
        start..end
    }
}

