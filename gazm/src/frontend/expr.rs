use grl_sources::Position;

use tower_lsp::lsp_types::lsif::ItemKind;
use unraveler::{
    all, alt, any, cut, is_a, many0, many1, many_until, match_item, not, opt, pair, preceded,
    sep_list, sep_pair, succeeded, tag, tuple, until, wrapped_cut, Collection, ParseError,
    ParseErrorKind, Parser, Severity,
};

use super::{
    concat, match_span, parse_label, parse_number, to_pos, IdentifierKind, NumberKind, PResult,
    ParseText, TSpan, Token, TokenKind,
};

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

pub fn parse_expr_list(input: TSpan) -> PResult<Vec<Node>> {
    sep_list(parse_expr, tag(TokenKind::Comma))(input)
}

pub fn parse_term(input: TSpan) -> PResult<Node> {
    alt((parse_unary_term, parse_non_unary_term))(input)
}

fn op_to_item(input: TSpan, toke: TokenKind, item: Item) -> PResult<Item> {
    let (rest, _) = tag(toke)(input)?;
    Ok((rest, item))
}

fn parse_unary_op(input: TSpan) -> PResult<Node> {
    use nom::combinator::map;
    let (rest, op) = alt((
        |i| op_to_item(i, TokenKind::Minus, Item::Sub),
        |i| op_to_item(i, TokenKind::GreaterThan, Item::UnaryGreaterThan),
    ))(input)?;

    let matched_span = to_pos(input);
    let node = Node::from_item_pos(op, matched_span);

    Ok((rest, node))
}

fn parse_pc(input: TSpan) -> PResult<Node> {
    let (rest, _) = op_to_item(input, TokenKind::Star, Item::Pc)?;
    let matched_span = to_pos(input);
    Ok((rest, Node::from_item_pos(Item::Pc, matched_span)))
}

fn parse_bracketed_expr(_input: TSpan) -> PResult<Node> {
    let (rest, mut matched) = preceded(tag(TokenKind::OpenBracket), parse_expr)(_input)?;
    let (rest, _) = tag(TokenKind::CloseBracket)(rest)?;
    let matched_span = to_pos(_input);
    matched.item = Item::BracketedExpr;
    Ok((rest, matched.with_pos(matched_span)))
}

pub fn parse_non_unary_term(input: TSpan) -> PResult<Node> {
    alt((parse_bracketed_expr, parse_number, parse_label, parse_pc))(input)
}

fn parse_unary_term(input: TSpan) -> PResult<Node> {
    let (rest, (op, term)) = pair(parse_unary_op, parse_non_unary_term)(input)?;

    let matched_span = to_pos(input);
    let node = Node::new_with_children(Item::UnaryTerm, &vec![op, term], matched_span);
    Ok((rest, node))
}

fn parse_binary_op(input: TSpan) -> PResult<Node> {
    use TokenKind::*;
    let (rest, op) = alt((
        |i| op_to_item(i, Plus, Item::Add),
        |i| op_to_item(i, LessThan, Item::Sub),
        |i| op_to_item(i, Star, Item::Mul),
        |i| op_to_item(i, Slash, Item::Div),
        |i| op_to_item(i, Bar, Item::BitOr),
        |i| op_to_item(i, Ampersand, Item::BitAnd),
        |i| op_to_item(i, Caret, Item::BitXor),
        |i| op_to_item(i, DoubleGreaterThan, Item::ShiftRight),
        |i| op_to_item(i, DoubleLessThan, Item::ShiftLeft),
    ))(input)?;

    let matched_span = to_pos(input);
    let node = Node::from_item_pos(op, matched_span);
    Ok((rest, node))
}

pub fn parse_op_term(input: TSpan) -> PResult<(Node, Node)> {
    let (rest, (op, term)) = pair(parse_binary_op, parse_term)(input)?;
    Ok((rest, (op, term)))
}

pub fn parse_expr(input: TSpan) -> PResult<Node> {
    let (rest, (matched_span, (term, vs))) =
        match_span(pair(parse_term, many0(parse_op_term)))(input)?;

    let vs = vs.into_iter().flat_map(|(o, t)| [o, t]);

    let node = Node::new_with_children(Item::Expr, &concat((term, vs)), to_pos(matched_span));

    Ok((rest, node))
}

#[cfg(test)]
mod test {
    use thin_vec::ThinVec;

    use super::super::*;
    use super::*;
    use Item::*;
    use ParsedFrom::*;

    fn get_children_items(node: &Node) -> ThinVec<Item> {
        node.children.iter().map(|c| c.item.clone()).collect()
    }

    #[test]
    fn test_expr() {
        let test = [
            (
                "3 * 4 + 0x1 + (10  + 4)",
                vec![
                    Number(3, Dec),
                    Mul,
                    Number(4, Dec),
                    Add,
                    Number(1, Hex),
                    Add,
                    BracketedExpr,
                ],
            ),
            ("-1 + -3", vec![UnaryTerm, Add, UnaryTerm]),
            ("1>>3", vec![Number(1, Dec), ShiftRight, Number(3, Dec)]),
        ];

        for (text, wanted) in test.iter() {
            println!("Parsing {text}");
            let tokens = to_tokens(&text);
            let span = tokens.as_slice().into();

            let (rest, matched) = parse_expr(span).unwrap();
            let items = get_children_items(&matched);

            assert!(rest.is_empty());
            assert_eq!(&items, wanted);
        }
    }
}
