/// parse opodes

use grl_sources::Position;

use unraveler::{
    all, alt, any, cut, is_a, many0, many1, many_until, match_item, not, opt, pair, preceded,
    sep_pair, succeeded, tag, tuple, until, wrapped_cut, Collection, ParseError, ParseErrorKind,
    Parser, Severity,
};

use super::{to_pos, IdentifierKind, NumberKind, PResult, TSpan, Token, TokenKind::{self,*}};

use crate::{
    async_tokenize::{GetTokensResult, IncludeErrorKind},
    item::{Item, LabelDefinition, Node, ParsedFrom},
};

pub fn parse_opode(_input: TSpan) -> PResult<Node> {
    panic!()
}
