// Parse expressions


use crate::{
    error::IResult,
    item::{Item, Node},
    item6809::MC6809,
};

use super::{
    util,
    labels::parse_label,
    locate::{matched_span, span_to_pos, Span},
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char as nom_char, multispace0},
    multi::many0,
    sequence::{preceded, separated_pair},
};

////////////////////////////////////////////////////////////////////////////////
// Operands
// so
// Addressing mode
//
//
//
/*
   Indexed,
   Direct,
   Extended, -> expr
   Relative, -> expr
   Relative16, -> expr
   Inherent,-> Nothing
   Immediate8, -> #expr
   Immediate16, -> #expr
*/

////////////////////////////////////////////////////////////////////////////////
// Expr Parsing
/// Parse a bracketed expression. Whitespace allowed before and after the
/// bracket
fn parse_bracketed_expr(input: Span) -> IResult<Node> {
    use util::{wrapped_chars, ws};
    let (rest, mut matched) = wrapped_chars('(', ws(parse_expr), ')')(input)?;
    let matched_span = matched_span(input, rest);
    matched.item = Item::BracketedExpr;
    Ok((rest, matched.with_span(matched_span)))
}

pub fn parse_pc(input: Span) -> IResult<Node> {
    let (rest, _matched) = nom_char('*')(input)?;
    let matched_span = matched_span(input, rest);
    Ok((rest, Node::from_item_span(Item::Pc, matched_span)))
}

pub fn parse_non_unary_term(input: Span) -> IResult<Node> {
    use util::parse_number;

    alt((parse_bracketed_expr, parse_number, parse_label, parse_pc))(input)
}

pub fn parse_term(input: Span) -> IResult<Node> {
    alt((parse_unary_term, parse_non_unary_term))(input)
}

fn parse_unary_term(input: Span) -> IResult<Node> {
    let (rest, (op, term)) =
        separated_pair(parse_unary_op, multispace0, parse_non_unary_term)(input)?;

    let matched_span = matched_span(input, rest);
    let node = Node::new_with_children(Item::UnaryTerm, &vec![op, term], span_to_pos(matched_span));
    Ok((rest, node))
}

fn parse_unary_op(input: Span) -> IResult<'_, Node> {
    use nom::combinator::map;
    let (rest, op) = alt((
        map(tag("-"), |_| Item::Sub),
        map(tag(">"), |_| Item::UnaryGreaterThan),
    ))(input)?;

    let matched_span = matched_span(input, rest);
    let node = Node::from_item_span(op, matched_span);

    Ok((rest, node))
}

fn parse_binary_op(input: Span) -> IResult<Node> {
    use nom::combinator::map;
    // let (rest, matched) = one_of(ops)(input)?;
    // let op = to_op(matched).unwrap();

    let (rest, op) = alt((
        map(tag("+"), |_| Item::Add),
        map(tag("-"), |_| Item::Sub),
        map(tag("*"), |_| Item::Mul),
        map(tag("/"), |_| Item::Div),
        map(tag("|"), |_| Item::BitOr),
        map(tag("&"), |_| Item::BitAnd),
        map(tag("^"), |_| Item::BitXor),
        map(tag(">>"), |_| Item::ShiftRight),
        map(tag("<<"), |_| Item::ShiftLeft),
    ))(input)?;

    let matched_span = matched_span(input, rest);
    let node = Node::from_item_span(op, matched_span);

    Ok((rest, node))
}

fn parse_op_term(input: Span) -> IResult<(Node, Node)> {
    let (rest, (op, term)) = separated_pair(parse_binary_op, multispace0, parse_term)(input)?;
    Ok((rest, (op, term)))
}

////////////////////////////////////////////////////////////////////////////////
pub fn parse_expr(input: Span) -> IResult<Node> {
    let (rest, term) = parse_term(input)?;
    let (rest, vs) = many0(preceded(multispace0, parse_op_term))(rest)?;

    let mut vec_ret = vec![term];
    vec_ret.extend(vs.into_iter().flat_map(|(o, t)| [o, t]));
    let matched_span = span_to_pos(matched_span(input, rest));
    let node = Node::new_with_children(Item::Expr, &vec_ret, matched_span);
    Ok((rest, node))
}

////////////////////////////////////////////////////////////////////////////////

#[allow(unused_imports)]
mod test {
    use super::*;
    use emu::utils::sources::AsmSource;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_brackets() {
        let input = "(10 + 4) + 20";
        let span = Span::new_extra(input, AsmSource::FromStr);
        let (rest, matched) = parse_bracketed_expr(span).unwrap();
        println!("{:#?}", matched);
        let matched = matched.to_string();
        assert_eq!(*rest, "+ 20");
        assert_eq!(matched, "(10+4)");
    }

    #[test]
    fn test_get_expr() {
        let input = "3 * 4 + %101 + -10";
        let span = Span::new_extra(input, AsmSource::FromStr);
        let (rest, matched) = parse_expr(span).unwrap();
        println!("{:#?}", matched);
        let matched = matched.to_string();

        assert_eq!(*rest, "");
        assert_eq!(matched, "3*4+%101+-10");

        let input = "3 * 4 + 5 - (5 * 4)";
        let span = Span::new_extra(input, AsmSource::FromStr);
        let (rest, matched) = parse_expr(span).unwrap();
        let matched = matched.to_string();

        assert_eq!(*rest, "");
        assert_eq!(matched, "3*4+5-(5*4)");
    }

    #[test]
    fn test_parse_pc() {
        let input = "* ;; ";
        let span = Span::new_extra(input, AsmSource::FromStr);
        let (rest, matched) = parse_pc(span).unwrap();
        assert_eq!(*rest, " ;; ");
        assert_eq!(&matched.to_string(), "*");
    }

    #[test]
    fn test_parse_pc2() {
        let input = "* * 3";
        let span = Span::new_extra(input, AsmSource::FromStr);
        let (_, matched) = parse_expr(span).unwrap();
        println!("{:#?}", matched);
        assert_eq!(&matched.to_string(), "**3");
    }
}
