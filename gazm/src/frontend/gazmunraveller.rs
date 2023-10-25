#![deny(unused_imports)]
use grl_sources::{Position, SourceFile, TextEditTrait};
use unraveler::Collection;

use super::{to_pos, Token, TokenKind};

#[derive(Clone, Copy, PartialEq)]
pub struct ParseState {}

impl Default for ParseState {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OriginalSource<'a> {
    pub source_file: &'a SourceFile,
}

impl<'a> OriginalSource<'a> {
    pub fn get_pos(&self, input: TSpan) -> Position {
        use grl_sources::AsmSource::FileId;

        let (s, e) = get_start_end_token(input);

        let extra_start = &e.extra;
        let extra_end = &s.extra;
        let r = extra_start.as_range().start..extra_end.as_range().end;
        let tp = &extra_start.pos;
        let file = extra_start.source_file.file_id;
        Position::new(tp.line(), tp.col(), r, FileId(file))
    }

    pub fn get_str(&self, input: TSpan) -> &str {
        let x = to_pos(input);
        let y = self.source_file.get_span(&x);
        y
    }
}

pub type TSpan<'a> = unraveler::Span<'a, Token<'a>, OriginalSource<'a>>;

pub struct Gaz {
    position: Position,
}

impl AsRef<Position> for Gaz {
    fn as_ref(&self) -> &Position {
        &self.position
    }
}

pub fn get_start_end_token<'a>(input: TSpan<'a>) -> (Token<'a>, Token<'a>) {
    if input.is_empty() {
        let doc = input.get_document();
        assert!(!doc.is_empty());
        let start = input.get_range().start;
        let toke = doc.get(start).or_else(|| doc.last()).unwrap();
        (toke.clone(), toke.clone())
    } else {
        let first = input.first().unwrap().clone();
        let last = input.last().unwrap().clone();
        (first, last)
    }
}

impl<'a> From<TSpan<'a>> for Gaz {
    fn from(_value: TSpan<'a>) -> Self {
        let te = _value.extra().source_file.as_text_edit();
        assert!(!te.is_empty());
        let (start, end) = get_start_end_token(_value);
        let _end = end.extra.as_range();
        let _start = start.extra.as_range();
        todo!()
    }
}
pub fn make_tspan<'a>(tokens: &'a [Token], sf: &'a grl_sources::SourceFile) -> TSpan<'a> {
    let span = TSpan::from_slice(&tokens, OriginalSource { source_file: sf });
    span
}

////////////////////////////////////////////////////////////////////////////////
impl unraveler::Item for Token<'_> {
    type Kind = TokenKind;

    fn get_kind(&self) -> Self::Kind {
        self.kind
    }
}

impl unraveler::Item for TokenKind {
    type Kind = TokenKind;

    fn get_kind(&self) -> Self::Kind {
        *self
    }
}

impl unraveler::Collection for TokenKind {
    type Item = TokenKind;

    fn at(&self, _index: usize) -> Option<&Self::Item> {
        Some(self)
    }

    fn length(&self) -> usize {
        1
    }
}
