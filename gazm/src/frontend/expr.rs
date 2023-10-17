use grl_sources::Position;

use itertools::Itertools;
use unraveler::{
    all, alt, any, cut, is_a, many0, many1, many_until, match_item, not, opt, pair, preceded,
    sep_list, sep_pair, succeeded, tuple, until, wrapped_cut, Collection, ParseError,
    ParseErrorKind, Parser, Severity,
};

use super::{
    concat, match_span as ms, parse_label, parse_number, to_pos, IdentifierKind, NumberKind,
    PResult, TSpan, Token,
    TokenKind::{self, *},
};

use crate::item::{Item, LabelDefinition, Node, ParsedFrom};

pub fn op_to_node(input: TSpan, toke: TokenKind, item: Item) -> PResult<Node> {
    let (rest, (sp, _)) = ms(toke)(input)?;
    Ok((rest, Node::from_item_pos(item, to_pos(sp))))
}

pub fn parse_expr_list(input: TSpan) -> PResult<Vec<Node>> {
    sep_list(parse_expr, Comma)(input)
}

pub fn parse_term(input: TSpan) -> PResult<Node> {
    alt((parse_unary_term, parse_non_unary_term))(input)
}

fn parse_unary_op(input: TSpan) -> PResult<Node> {
    alt((
        |i| op_to_node(i, Minus, Item::Sub),
        |i| op_to_node(i, GreaterThan, Item::UnaryGreaterThan),
    ))(input)
}

fn parse_bracketed_expr(input: TSpan) -> PResult<Node> {
    let (rest, (sp, mut matched)) = ms(wrapped_cut(OpenBracket, parse_expr, CloseBracket))(input)?;
    matched.item = Item::BracketedExpr;
    Ok((rest, matched.with_pos(to_pos(sp))))
}

pub fn parse_non_unary_term(input: TSpan) -> PResult<Node> {
    let parse_pc = |i| op_to_node(i, Star, Item::Pc);
    alt((parse_bracketed_expr, parse_number, parse_label, parse_pc))(input)
}

fn parse_unary_term(input: TSpan) -> PResult<Node> {
    let (rest, (matched_span, (op, term))) = ms(
        pair(parse_unary_op, parse_non_unary_term)
    )(input)?;
    let node = Node::new_with_children(Item::UnaryTerm, &vec![op, term], to_pos(matched_span));
    Ok((rest, node))
}

fn parse_binary_op(input: TSpan) -> PResult<Node> {
    alt((
        |i| op_to_node(i, Plus, Item::Add),
        |i| op_to_node(i, LessThan, Item::Sub),
        |i| op_to_node(i, Star, Item::Mul),
        |i| op_to_node(i, Slash, Item::Div),
        |i| op_to_node(i, Bar, Item::BitOr),
        |i| op_to_node(i, Ampersand, Item::BitAnd),
        |i| op_to_node(i, Caret, Item::BitXor),
        |i| op_to_node(i, DoubleGreaterThan, Item::ShiftR),
        |i| op_to_node(i, DoubleLessThan, Item::ShiftL),
    ))(input)
}

pub fn parse_op_term(input: TSpan) -> PResult<(Node, Node)> {
    let (rest, (op, term)) = pair(parse_binary_op, parse_term)(input)?;
    Ok((rest, (op, term)))
}

pub fn parse_expr(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (term, vs))) = ms(pair(parse_term, many0(parse_op_term)))(input)?;

    if vs.is_empty() && term.item.is_number() {
        Ok((rest, term))
    } else {
        let vs = vs.into_iter().flat_map(|(o, t)| [o, t]);
        let node = Node::new_with_children(Item::Expr, &concat((term, vs)), to_pos(sp));
        Ok((rest, node))
    }
}

#[cfg(test)]
mod test {
    use thin_vec::ThinVec;

    use super::super::*;
    use crate::item::{
        Item::{self, *},
        Node,
        ParsedFrom::*,
    };

    #[test]
    fn test_expr() {
        let test = [
            ("3", Num(3, Dec), vec![]),
            (
                "3 * 4 + 0x1 + (10  + 4)",
                Item::Expr,
                vec![
                    Num(3, Dec),
                    Mul,
                    Num(4, Dec),
                    Add,
                    Num(1, Hex),
                    Add,
                    BracketedExpr,
                ],
            ),
            ("-1 + -3", Expr, vec![UnaryTerm, Add, UnaryTerm]),
            ("1>>3", Expr, vec![Num(1, Dec), ShiftR, Num(3, Dec)]),
        ];

        for (text, i, wanted) in test.iter() {
            println!("Parsing {text}");
            let source_file = create_source_file(text);
            let tokens = to_tokens(&source_file);
            let span = TSpan::from_slice(&tokens, Default::default());
            let (rest, matched) = parse_expr(span).unwrap();
            let (item, items) = get_items(&matched);
            println!("\tItem: {:?} : {:?}", item, items);
            assert_eq!(&item, i);
            assert!(rest.is_empty());
            assert_eq!(&items, wanted);
        }
    }
}
