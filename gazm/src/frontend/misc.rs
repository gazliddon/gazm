#![deny(unused_imports)]
use unraveler::{
    alt, cut, map, match_item, match_span as ms, preceded, sep_list, sep_pair, tag, wrapped_cut,
    Parser,
};

use crate::assembler::AssemblerCpuTrait;

use super::{
    get_text, CommandKind, FrontEndError, GazmParser, Item, LabelDefinition, Node,
    NumberKind, PResult, ParsedFrom, TSpan, Token, TokenKind
};

fn match_number(input: TSpan) -> PResult<(TSpan, TokenKind)> {
    use TokenKind::Number;
    let (rest, (sp, matched)) = ms(match_item(|i: &Token| matches!(i.kind, Number(..))))(input)?;
    Ok((rest, (sp, matched.kind)))
}

impl<C> GazmParser<C>
where
    C: AssemblerCpuTrait,
{
    pub fn parse_number(input: TSpan) -> PResult<Node<C::NodeKind>> {
    use TokenKind::Number;
        let (rest, (sp, kind)) = match_number(input)?;

        match kind {
            Number((n, nk)) => {
                let node = Self::from_item_tspan(Item::Num(n, nk.into()), sp);
                Ok((rest, node))
            }
            _ => panic!(),
        }
    }

    pub(crate) fn get_label<F: Fn(String) -> LabelDefinition>(
        input: TSpan,
        mut tag_kind: TokenKind,
        to_label_def: F,
    ) -> PResult<Node<C::NodeKind>> {
        let (rest, sp) = tag_kind.parse(input)?;
        let node = Self::from_item_tspan(Item::Label(to_label_def(get_text(sp))), sp);
        Ok((rest, node))
    }

    fn parse_local_label(input: TSpan) -> PResult<Node<C::NodeKind>> {
        
        use TokenKind::{Pling,At};
        use { Item::LocalLabel, LabelDefinition::Text};
        let (rest, (sp, matched)) = ms(preceded(alt((Pling, At)), TokenKind::Label))(input)?;

        let label_def = Text(get_text(matched));
        let node = Self::from_item_tspan(LocalLabel(label_def), sp);
        Ok((rest, node))
    }

    pub fn parse_non_scoped_label(input: TSpan) -> PResult<Node<C::NodeKind>> {
        use { LabelDefinition::Text, TokenKind::Label};
        Self::get_label(input, Label, Text)
    }

    pub fn parse_scoped_label(input: TSpan) -> PResult<Node<C::NodeKind>> {
    use TokenKind::FqnIdentifier;
        use LabelDefinition::TextScoped;
        Self::get_label(input, FqnIdentifier, TextScoped)
    }

    pub fn parse_label(input: TSpan) -> PResult<Node<C::NodeKind>> {
        alt((
            Self::parse_local_label,
            Self::parse_scoped_label,
            Self::parse_non_scoped_label,
        ))(input)
    }

    pub fn parse_label_assignment_pc(input: TSpan) -> PResult<Node<C::NodeKind>> {
        alt((
            Self::parse_local_label,
            Self::parse_scoped_label,
            Self::parse_non_scoped_label,
        ))(input)
    }
    pub fn parse_big_import(input: TSpan) -> PResult<Node<C::NodeKind>> {
        use TokenKind::{OpenBrace,CloseBrace, Comma};
        use CommandKind::Import;
        let (rest, (span, matched)) = ms(preceded(
            Import,
            wrapped_cut(
                OpenBrace,
                sep_list(Self::parse_scoped_label, Comma),
                CloseBrace,
            ),
        ))(input)?;
        let node = Self::from_item_kids_tspan(Item::Import, &matched, span);
        Ok((rest, node))
    }
}

impl<'a> Parser<TSpan<'a>, TSpan<'a>, FrontEndError> for CommandKind {
    fn parse(&mut self, i: TSpan<'a>) -> Result<(TSpan<'a>, TSpan<'a>), FrontEndError> {
        TokenKind::Command(*self).parse(i)
    }
}

impl<'a> Parser<TSpan<'a>, TSpan<'a>, FrontEndError> for TokenKind {
    fn parse(&mut self, i: TSpan<'a>) -> Result<(TSpan<'a>, TSpan<'a>), FrontEndError> {
        tag(*self)(i)
    }
}

impl From<NumberKind> for ParsedFrom {
    fn from(nk: NumberKind) -> Self {
        match nk {
            NumberKind::Char => ParsedFrom::Character,
            NumberKind::Hex => ParsedFrom::Hexadecimal,
            NumberKind::Dec => ParsedFrom::Decimal,
            NumberKind::Bin => ParsedFrom::Binary,
        }
    }
}

impl<C> GazmParser<C>
where
    C: AssemblerCpuTrait,
{
    fn get_label_definition(item: &Item<C::NodeKind>) -> Option<LabelDefinition> {
        match item {
            Item::Label(l) | Item::LocalLabel(l) => Some(l.clone()),
            _ => None,
        }
    }

    fn parse_local_assignment(input: TSpan) -> PResult<Item<C::NodeKind>>
    where
        C: std::fmt::Debug + Clone + PartialEq,
    {
        use Item::LocalAssignment;
        map(Self::parse_local_label, |e| {
            LocalAssignment(Self::get_label_definition(&e.item).unwrap())
        })(input)
    }

    fn parse_assignment(input: TSpan) -> PResult<Item<C::NodeKind>>
    where
        C: std::fmt::Debug + Clone + PartialEq,
    {
        use Item::Assignment;
        map(Self::parse_label, |e| {
            Assignment(Self::get_label_definition(&e.item).unwrap())
        })(input)
    }

    pub fn parse_equate(input: TSpan) -> PResult<Node<C::NodeKind>> {
        let command =  TokenKind::Command(CommandKind::Equ);
        let (rest, (sp, (assignment, expr))) = ms(sep_pair(
            alt((Self::parse_local_assignment, Self::parse_assignment)),
            tag(command),
            cut(Self::parse_expr),
        ))(input)?;

        let node = Self::from_item_kid_tspan(assignment, expr, sp);
        Ok((rest, node))
    }
}
