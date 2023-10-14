use itertools::Itertools;
use unraveler::{
    all, alt, any, cut, is_a, many0, many1, many_until, match_item, not, opt, pair, preceded,sep_list,
    sep_pair, succeeded, tag, tuple, until, wrapped, wrapped_cut, Collection, ParseError,
    ParseErrorKind, Parser, Severity, Splitter,
};

use thin_vec::{thin_vec, ThinVec};

use super::{
    command, to_pos, IdentifierKind, NumberKind, PResult, ParseText, TSpan, Token, TokenKind,
    match_span,
};

use crate::{
    async_tokenize::{GetTokensResult, IncludeErrorKind},
    cli::parse,
    error::IResult,
    frontend::CommandKind,
    item::{Item, LabelDefinition, Node, ParsedFrom},
    item6809::{IndexParseType, MC6809::SetDp},
    parse::{
        locate::{matched_span, span_to_pos},
        util::match_str,
    },
};

pub fn parse_number(input: TSpan) -> PResult<Node> {
    let pred = |i: &Token<ParseText>| matches!(i.kind, TokenKind::Number(..));

    let (rest, matched) = match_item(pred)(input)?;
    let matched_pos = to_pos(input);

    match matched.kind {
        TokenKind::Number((n, nk)) | TokenKind::Char((n, nk)) => {
            let pf = match nk {
                NumberKind::Char => ParsedFrom::Char,
                NumberKind::Hex => ParsedFrom::Hex,
                NumberKind::Dec => ParsedFrom::Dec,
                NumberKind::Bin => ParsedFrom::Bin,
            };
            let node = Node::new(Item::Number(n, pf), matched_pos);
            Ok((rest, node))
        }
        _ => panic!(),
    }
}

fn get_label<F: Fn(&str) -> LabelDefinition>(
    input: TSpan,
    tag_kind: TokenKind,
    to_label_def: F,
) -> PResult<Node> {
    let (rest, matched) = tag(tag_kind)(input)?;
    let matched_pos = to_pos(input);
    let label_def = to_label_def(matched.first().unwrap().extra.get_text());
    let item = Item::Label(label_def);
    let node = Node::new(item, matched_pos);
    Ok((rest, node))
}

fn parse_local_label(input: TSpan) -> PResult<Node> {
    let (rest, matched) = preceded(
        alt((tag(TokenKind::Pling), tag(TokenKind::At))),
        tag(TokenKind::Identifier(IdentifierKind::Label)),
    )(input)?;
    let matched_pos = to_pos(input);
    let label_def = LabelDefinition::Text(matched.first().unwrap().extra.get_text().to_owned());
    let node = Node::new(Item::LocalLabel(label_def), matched_pos);
    Ok((rest, node))
}

fn parse_non_scoped_label(input: TSpan) -> PResult<Node> {
    let tag_kind = TokenKind::Identifier(IdentifierKind::Label);
    let to_label_def = |text: &str| LabelDefinition::Text(text.to_owned());
    get_label(input, tag_kind, to_label_def)
}

pub fn parse_scoped_label(input: TSpan) -> PResult<Node> {
    let tag_kind = TokenKind::Identifier(IdentifierKind::Label);
    let to_label_def = |text: &str| LabelDefinition::TextScoped(text.to_owned());
    get_label(input, tag_kind, to_label_def)
}

pub fn parse_label(input: TSpan) -> PResult<Node> {
    alt((
        parse_local_label,
        parse_scoped_label,
        parse_non_scoped_label,
    ))(input)
}

pub fn parse_big_import(input: TSpan) -> PResult<Node> {
    use TokenKind::*;

    let parser = preceded(
        command(CommandKind::Import),
        wrapped(
            OpenBrace,
            sep_list(parse_scoped_label, tag(Comma)),
            CloseBrace,
        ),
    );

    let (rest, (span, matched)) = match_span(parser)(input)?;
    let node = Node::new_with_children(Item::Import, &matched, to_pos(span));
    Ok((rest, node))
}

pub fn parse_import(input: TSpan) -> PResult<Node> {
    let (rest, scoped) = preceded(command(CommandKind::Import), parse_scoped_label)(input)?;
    let matched_pos = to_pos(input);
    let node = Node::new_with_children(Item::Import, &[scoped], matched_pos);
    Ok((rest, node))
}
