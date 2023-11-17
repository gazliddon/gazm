#![deny(unused_imports)]
use grl_sources::{Position, SourceFile};
use unraveler::Collection;
use crate::opts::Opts;
use super::{to_pos, Token, TokenKind};

#[derive(Default,Clone, Copy, PartialEq)]
pub struct ParseState {}

#[derive(Copy, Clone, Debug)]
pub struct OriginalSource<'a> {
    pub source_file: &'a SourceFile,
    pub is_parsing_macro_def : bool,
    pub opts : &'a Opts,
}

impl<'a> OriginalSource<'a> {
    pub fn get_pos(&self, input: TSpan) -> Position {
        let (s, e) = get_start_end_token(input);
        get_start_end_position(&s, &e)
    }

    pub fn get_str(&self, input: TSpan) -> &str {
        let x = to_pos(input);
        let y = self.source_file.get_span(&x);
        y
    }

    pub fn set_macro(&mut self, v : bool) {
        self.is_parsing_macro_def = v
    }
}

pub type TSpan<'a> = unraveler::Span<'a, Token<'a>, OriginalSource<'a>>;

pub fn get_start_end_position(s: &Token, e: &Token) -> Position {
    let extra_start = &s.extra;
    let extra_end = &e.extra;

    let r = extra_start.as_range().start..extra_end.as_range().end;
    let tp = &extra_start.pos;
    let file_id = extra_start.source_file.file_id;
    Position::new(tp.line(), tp.col(), r, file_id)
}

pub fn get_start_end_token(input: TSpan) -> (Token, Token) {
    if input.is_empty() {
        let doc = input.get_document();
        assert!(!doc.is_empty());
        let start = input.offset();
        let toke = doc.get(start).or_else(|| doc.last()).unwrap();
        (*toke, *toke)
    } else {
        let first = input.first().unwrap();
        let last = input.last().unwrap();
        (*first, *last)
    }
}

pub fn make_tspan<'a>(tokens: &'a [Token], sf: &'a grl_sources::SourceFile, opts: &'a Opts) -> TSpan<'a> {
    let span = TSpan::from_slice(tokens, OriginalSource { source_file: sf, is_parsing_macro_def : false, opts });
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
