use grl_sources::Position;

use unraveler::{
    all, alt, cut, is_a, many0, many1, many_until, not, opt, pair, preceded, sep_pair, succeeded,
    tag, tuple, until, wrapped_cut, Collection, ParseError, ParseErrorKind, Parser, Severity,
};

use super::{PResult,TSpan, TokenKind};


use crate::{
    async_tokenize::IncludeErrorKind,
    cli::parse,
    error::IResult,
    item::{Item, LabelDefinition, Node, ParsedFrom},
    item6809::{IndexParseType, MC6809::SetDp},
    parse::locate::span_to_pos,
};

pub fn parse_expr(_input: TSpan) -> PResult<Node> {
    panic!()
}

pub fn parse_expr_list(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, (x,mut xs)) = pair(parse_expr, many0(preceded(tag(TokenKind::Comma), parse_expr)))(input)?;
    let mut ret = vec![x];
    ret.append(&mut xs);
    Ok((rest,xs))
}
