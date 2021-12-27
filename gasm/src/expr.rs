
// Parse expressions
//
use super::item::{ Item,Node };
use nom::error::Error;
use nom::error::ErrorKind::NoneOf;
use nom::character::complete::{multispace0, char as nom_char, one_of };

use nom::bytes::complete::{
    is_a, tag,
};

use nom::sequence::{separated_pair, terminated};

use nom::branch::alt;
use nom::multi::many0;
use nom::combinator::recognize;

use super::util;

use super::labels::parse_label;

use crate::error::{IResult, ParseError};
use crate::locate::Span;

////////////////////////////////////////////////////////////////////////////////
// Operands
// so 
// Addressing mode
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

fn parse_bracketed_expr(input: Span) -> IResult< Node> {
    let (rest, matched) = util::wrapped_chars('(', parse_expr, ')')(input)?;
    Ok(( rest, matched.with_pos(input, rest) ))
}

fn parse_pc(input : Span) -> IResult< Node> {
    let (rest, _matched) = nom_char('*')(input)?;
    Ok((rest, Node::from_item(Item::Pc).with_pos(input, rest)))
}

fn parse_non_unary_term(input: Span) -> IResult< Node> {
    use util::parse_number;

    alt((parse_bracketed_expr,
          parse_number,
          parse_label,
          parse_pc,
          ))(input)
}

pub fn parse_term(input: Span) -> IResult< Node> {
    alt((parse_unary_term, parse_non_unary_term))(input)
}

fn parse_unary_term(input: Span) -> IResult< Node> {
    use util::parse_number;
    let (rest, (op, term)) = separated_pair(parse_unary_op,  multispace0, parse_term)(input)?;
    let ret = Node::from_item(Item::UnaryTerm)
        .with_children(vec![op,term])
        .with_pos(input, rest);
    Ok((rest, ret))
}

fn parse_unary_op(input: Span) -> IResult< Node> {
    let ops = "+-";

    let (rest, matched) = one_of(ops)(input)?;

    let op = match matched {
        '+' => Item::UnaryPlus,
        '-' => Item::UnaryMinus,
        _ => panic!("{:?}", matched),
    };

    let ret = Node::from_item(op).with_pos(input, rest);
    Ok((rest, ret))
}

fn parse_op(input: Span) -> IResult< Node> {
    let ops = "+-*/";

    let (rest, matched) = one_of(ops)(input)?;

    let op = match matched {
        '+' => Item::Add,
        '-' => Item::Sub,
        '*' => Item::Mul,
        '/' => Item::Div,
        _ => panic!("{:?}", matched),
    };
    let ret = Node::from_item(op).with_pos(input, rest);
    Ok((rest, ret))
}

fn parse_op_term(input: Span) -> IResult< Node> {
    let (rest, (op, term)) = separated_pair(parse_op, multispace0, parse_term)(input)?;
    let node = op.with_child(term).with_pos(input,rest);
    Ok((rest,node))
}

fn prepend(i : Node, is : Vec<Node>) -> Vec<Node> {
    let mut ret = vec![i];
    ret.extend(is);
    ret
}

////////////////////////////////////////////////////////////////////////////////
pub fn parse_expr(input: Span) -> IResult<Node> {
    let (rest, (v,vs)) = separated_pair(parse_term, multispace0, many0(parse_op_term))(input)?;

    let node = if vs.is_empty() {
        v
    } else {
        let v = prepend(v,vs);
        Node::from_item(Item::Expr).with_children(v)
    };

    let node = node.with_pos(input,rest);

    Ok(( rest,node ))
}

////////////////////////////////////////////////////////////////////////////////

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_expr() {

    }
}
