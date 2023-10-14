use unraveler::{
    all, alt, any, cut, is_a, many0, many1, many_until, match_item, not, opt, pair, preceded,
    sep_list, sep_pair, succeeded, tag, tuple, until, wrapped, wrapped_cut, Collection, ParseError,
    ParseErrorKind, Parser, Severity, Splitter,
};

use super::{
    get_str, get_text, match_span as ms, to_pos, CommandKind, IdentifierKind, MyError, NumberKind,
    PResult, ParseText, TSpan, Token, TokenKind::{self,*}
};

use crate::item::{Item, LabelDefinition, Node, ParsedFrom};

fn match_number(input: TSpan) -> PResult<(TSpan, TokenKind)> {
    let (rest, (sp, matched)) = ms(match_item(|i: &Token| {
        matches!(i.kind, Number(..))
    }))(input)?;
    Ok((rest, (sp, matched.kind)))
}

pub fn parse_number(input: TSpan) -> PResult<Node> {

    let (rest, (sp, kind)) = match_number(input)?;

    match kind {
        Number((n, nk)) | Char((n, nk)) => {
            let node = Node::new(Item::Number(n, nk.into()), to_pos(sp));
            Ok((rest, node))
        }
        _ => panic!(),
    }
}

fn get_label<F: Fn(String) -> LabelDefinition>(
    input: TSpan,
    mut tag_kind: TokenKind,
    to_label_def: F,
) -> PResult<Node> {
    let (rest, sp) = tag_kind.parse(input)?;
    let node = Node::new(Item::Label(to_label_def(get_text(sp))), to_pos(sp));
    Ok((rest, node))
}

fn parse_local_label(input: TSpan) -> PResult<Node> {
    use {IdentifierKind::*, Item::LocalLabel, LabelDefinition::Text} ;
    let (rest, (sp, matched)) = ms(preceded(alt((Pling, At)), Identifier(Label)))(input)?;
    let label_def = Text(get_text(matched));
    let node = Node::new(LocalLabel(label_def), to_pos(sp));
    Ok((rest, node))
}

fn parse_non_scoped_label(input: TSpan) -> PResult<Node> {
    use {IdentifierKind::*, LabelDefinition::Text};
    get_label(input, Identifier(Label), Text)
}

pub fn parse_scoped_label(input: TSpan) -> PResult<Node> {
    use {IdentifierKind::*, LabelDefinition::TextScoped};
    get_label(input, Identifier(Label), TextScoped)
}

pub fn parse_label(input: TSpan) -> PResult<Node> {
    alt((
        parse_local_label,
        parse_scoped_label,
        parse_non_scoped_label,
    ))(input)
}

impl<'a> Parser<TSpan<'a>, TSpan<'a>, MyError> for CommandKind {
    fn parse(&mut self, i: TSpan<'a>) -> Result<(TSpan<'a>, TSpan<'a>), MyError> {
        TokenKind::Identifier(IdentifierKind::Command(*self)).parse(i)
    }
}

impl<'a> Parser<TSpan<'a>, TSpan<'a>, MyError> for TokenKind {
    fn parse(&mut self, i: TSpan<'a>) -> Result<(TSpan<'a>, TSpan<'a>), MyError> {
        use IdentifierKind::*;
        tag(*self)(i)
    }
}

pub fn parse_big_import(input: TSpan) -> PResult<Node> {
    use CommandKind::Import;

    let (rest, (span, matched)) = ms(preceded(
        Import,
        wrapped_cut(OpenBrace, sep_list(parse_scoped_label, Comma), CloseBrace),
    ))(input)?;
    let node = Node::new_with_children(Item::Import, &matched, to_pos(span));
    Ok((rest, node))
}

pub fn parse_import(input: TSpan) -> PResult<Node> {
    let (rest, scoped) = preceded(CommandKind::Import, parse_scoped_label)(input)?;
    let matched_pos = to_pos(input);
    let node = Node::new_with_children(Item::Import, &[scoped], matched_pos);
    Ok((rest, node))
}

impl From<NumberKind> for ParsedFrom {
    fn from(nk: NumberKind) -> Self {
        match nk {
            NumberKind::Char => ParsedFrom::Char,
            NumberKind::Hex => ParsedFrom::Hex,
            NumberKind::Dec => ParsedFrom::Dec,
            NumberKind::Bin => ParsedFrom::Bin,
        }
    }
}
