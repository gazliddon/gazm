/// parse opodes

use grl_sources::Position;

use tower_lsp::lsp_types::lsif::ItemKind;
use unraveler::{
    all, alt, any, cut, is_a, many0, many1, many_until, match_item, not, opt, pair, preceded,
    sep_pair, succeeded, tag, tuple, until, wrapped_cut, Collection, ParseError, ParseErrorKind,
    Parser, Severity,
};

use super::{to_pos, IdentifierKind, NumberKind, PResult, ParseText, TSpan, Token, TokenKind};

use crate::{
    async_tokenize::{GetTokensResult, IncludeErrorKind},
    cli::parse,
    error::IResult,
    item::{Item, LabelDefinition, Node, ParsedFrom},
    item6809::{IndexParseType, MC6809::SetDp},
    parse::{
        locate::{matched_span, span_to_pos},
        util::match_str,
    },
};

pub fn parse_opode(_input: TSpan) -> PResult<Node> {
    panic!()
}
