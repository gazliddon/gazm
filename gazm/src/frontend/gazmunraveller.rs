#![deny(unused_imports)]
use super::{to_pos, Token, TokenKind};
use crate::{cpukind::CpuKind, opts::Opts};
use grl_sources::{Position, SourceFile};
use unraveler::Collection;

#[derive(Copy, Clone, Debug)]
pub struct ParseContext<'a> {
    pub source_file: &'a SourceFile,
    pub is_parsing_macro_def: bool,
    pub cpu_kind: Option<CpuKind>,
    pub opts: &'a Opts,
}

impl<'a> ParseContext<'a> {
    pub fn get_pos(&self, input: TSpan) -> Position {
        if let Some((s, e)) = get_start_end_token(input) {
            return get_start_end_position(&s, &e);
        }

        // An empty token document has no source token to anchor to. Keep the
        // diagnostic representable without panicking; normal lexed input is
        // non-empty, so this is only a defensive fallback.
        let offset = input.offset();
        Position::new(0, 0, offset..offset, self.source_file.file_id)
    }

    pub fn get_str(&self, input: TSpan) -> &str {
        let x = to_pos(input);
        let y = self.source_file.get_span(&x);
        y
    }

    pub fn set_macro(&mut self, v: bool) {
        self.is_parsing_macro_def = v
    }

    pub fn set_cpu_kind(&mut self, v: CpuKind) {
        self.cpu_kind = Some(v)
    }
}

pub type TSpan<'a> = unraveler::Span<'a, Token<'a>, ParseContext<'a>>;

pub fn get_start_end_position(s: &Token, e: &Token) -> Position {
    let extra_start = &s.extra;
    let extra_end = &e.extra;

    let r = extra_start.as_range().start..extra_end.as_range().end;
    let tp = &extra_start.pos;
    let file_id = extra_start.source_file.file_id;
    Position::new(tp.line(), tp.col(), r, file_id)
}

pub fn get_start_end_token(input: TSpan) -> Option<(Token, Token)> {
    if input.is_empty() {
        let doc = input.get_document();
        let start = input.offset();
        let token = doc.get(start).or_else(|| doc.last())?;
        Some((*token, *token))
    } else {
        Some((*input.first()?, *input.last()?))
    }
}

pub fn make_tspan<'a>(
    tokens: &'a [Token],
    sf: &'a grl_sources::SourceFile,
    opts: &'a Opts,
) -> TSpan<'a> {
    let span = TSpan::from_slice(
        tokens,
        ParseContext {
            source_file: sf,
            is_parsing_macro_def: false,
            opts,
            cpu_kind: Some(opts.cpu),
        },
    );
    span
}

////////////////////////////////////////////////////////////////////////////////
impl unraveler::Item for Token<'_> {
    type Kind = TokenKind;

    fn get_kind(&self) -> Self::Kind {
        self.kind
    }
}
