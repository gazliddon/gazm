#![deny(unused_imports)]
use unraveler::{
    alt, cut, delimited_kind, kind, map, match_item, match_span as ms, preceded, sep_list, tuple,
    Parser,
};

use super::{
    from_item_child_tspan, from_item_children_tspan, from_item_tspan, get_text, keyword,
    parse_expr, AstNodeKind, CommandKind, FrontEndError, GazmParser, LabelDefinition, Node,
    NumberKind, PResult, ParsedFrom, TSpan, Token, TokenKind,
};

fn match_number(input: TSpan) -> PResult<(TSpan, TokenKind)> {
    use TokenKind::Number;
    let (rest, (sp, matched)) = ms(match_item(|i: &Token| matches!(i.kind, Number(..))))(input)?;
    Ok((rest, (sp, matched.kind)))
}

impl GazmParser {
    pub fn parse_number(input: TSpan) -> PResult<Node> {
        use TokenKind::Number;
        let (rest, (sp, kind)) = match_number(input)?;

        match kind {
            Number((n, nk)) => {
                let node = from_item_tspan(AstNodeKind::Num(n, nk.into()), sp);
                Ok((rest, node))
            }
            _ => panic!(),
        }
    }

    pub(crate) fn get_label<F: Fn(String) -> LabelDefinition>(
        input: TSpan,
        mut tag_kind: TokenKind,
        to_label_def: F,
    ) -> PResult<Node> {
        let (rest, sp) = tag_kind.parse(input)?;
        let node = from_item_tspan(AstNodeKind::Label(to_label_def(get_text(sp))), sp);
        Ok((rest, node))
    }

    pub(crate) fn parse_local_label(input: TSpan) -> PResult<Node> {
        use TokenKind::{At, LocalIdentifier, Pling};
        use {AstNodeKind::LocalLabel, LabelDefinition::Text};
        if let Ok((rest, span)) = LocalIdentifier.parse(input) {
            let text = get_text(span);
            return Ok((
                rest,
                from_item_tspan(LocalLabel(Text(text[1..].to_owned())), span),
            ));
        }
        let (rest, (sp, matched)) = ms(preceded(
            alt((Pling, At)),
            alt((TokenKind::Label, TokenKind::Identifier)),
        ))(input)?;

        let label_def = Text(get_text(matched));
        let node = from_item_tspan(LocalLabel(label_def), sp);
        Ok((rest, node))
    }

    pub fn parse_non_scoped_label(input: TSpan) -> PResult<Node> {
        use LabelDefinition::Text;
        let (rest, sp) = alt((TokenKind::Label, TokenKind::Identifier))(input)?;
        Ok((
            rest,
            from_item_tspan(AstNodeKind::Label(Text(get_text(sp))), sp),
        ))
    }

    pub fn parse_scoped_label(input: TSpan) -> PResult<Node> {
        use LabelDefinition::TextScoped;
        use TokenKind::FqnIdentifier;
        Self::get_label(input, FqnIdentifier, TextScoped)
    }

    pub fn parse_label(input: TSpan) -> PResult<Node> {
        alt((
            Self::parse_local_label,
            Self::parse_scoped_label,
            Self::parse_non_scoped_label,
        ))(input)
    }

    pub fn parse_label_assignment_pc(input: TSpan) -> PResult<Node> {
        alt((
            Self::parse_local_label,
            Self::parse_scoped_label,
            Self::parse_non_scoped_label,
        ))(input)
    }
    pub fn parse_big_import(input: TSpan) -> PResult<Node> {
        use CommandKind::Import;
        use TokenKind::{CloseBrace, Comma, OpenBrace};
        let (rest, (span, matched)) = ms(preceded(
            Import,
            delimited_kind(
                OpenBrace,
                sep_list(Self::parse_scoped_label, Comma),
                CloseBrace,
            ),
        ))(input)?;
        let node = from_item_children_tspan(AstNodeKind::Import, &matched, span);
        Ok((rest, node))
    }
}

impl<'a> Parser<TSpan<'a>, TSpan<'a>, FrontEndError> for CommandKind {
    fn parse(&mut self, i: TSpan<'a>) -> Result<(TSpan<'a>, TSpan<'a>), FrontEndError> {
        // Match the command's keyword by text, not as a reserved token:
        // a word is a command only where a parser chooses to match it, so
        // user identifiers with the same spelling keep working elsewhere.
        keyword(self.keyword_name())(i)
    }
}

impl<'a> Parser<TSpan<'a>, TSpan<'a>, FrontEndError> for TokenKind {
    fn parse(&mut self, i: TSpan<'a>) -> Result<(TSpan<'a>, TSpan<'a>), FrontEndError> {
        kind(*self)(i)
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

impl GazmParser {
    fn get_label_definition(item: &AstNodeKind) -> Option<LabelDefinition> {
        match item {
            AstNodeKind::Label(l) | AstNodeKind::LocalLabel(l) => Some(l.clone()),
            _ => None,
        }
    }

    fn parse_local_assignment(input: TSpan) -> PResult<AstNodeKind> {
        use AstNodeKind::LocalAssignment;
        map(Self::parse_local_label, |e| {
            LocalAssignment(Self::get_label_definition(&e.item).unwrap())
        })(input)
    }

    fn parse_assignment(input: TSpan) -> PResult<AstNodeKind> {
        use AstNodeKind::Assignment;
        map(Self::parse_label, |e| {
            Assignment(Self::get_label_definition(&e.item).unwrap())
        })(input)
    }

    pub fn parse_equate(input: TSpan) -> PResult<Node> {
        let (rest, (sp, (assignment, _, _, expr))) = ms(tuple((
            alt((Self::parse_local_assignment, Self::parse_assignment)),
            kind(TokenKind::Colon),
            keyword("equ"),
            cut(parse_expr),
        )))(input)?;

        let node = from_item_child_tspan(assignment, expr, sp);
        Ok((rest, node))
    }
}
