


// Parse expressions
//
use super::item::{ Item,Node };
use nom::error::Error;
use nom::error::ErrorKind::NoneOf;
use nom::character::complete::{multispace0, char as nom_char, one_of };

use nom::InputTake;

use nom::bytes::complete::{
    is_a, tag,
};

use nom::sequence::{preceded, separated_pair, terminated};

use nom::branch::alt;
use nom::multi::many0;
use nom::combinator::{map_parser, recognize};

use super::util;

use super::labels::parse_label;

use crate::error::{IResult, ParseError};
use crate::locate::{Span, matched_span};
use crate::opcodes::parse_opcode;

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
    let (rest, mut matched) = util::wrapped_chars('(', parse_expr, ')')(input)?;
    let matched_span = matched_span(input, rest);
    matched.item = Item::BracketedExpr;
    Ok(( rest, matched.with_ctx(matched_span) ))
}

pub fn parse_pc(input : Span) -> IResult< Node> {
    let (rest, _matched) = nom_char('*')(input)?;
    let matched_span = matched_span(input, rest);
    Ok((rest, Node::from_item(Item::Pc, matched_span)))
}

pub fn parse_non_unary_term(input: Span) -> IResult< Node> {
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
    let (rest, (op, term)) = separated_pair(parse_unary_op,  multispace0, parse_non_unary_term)(input)?;

    let matched_span = matched_span(input, rest);
    let ret = Node::from_item(Item::UnaryTerm, matched_span)
        .with_children(vec![op,term]) ;
    Ok((rest, ret))
}

fn parse_op_allowed<'a>(input: Span<'a>,ops : &str) -> IResult<'a, Node> {
    let (rest, matched) = one_of(ops)(input)?;

    let op=to_op(matched).unwrap();

    let matched_span = matched_span(input, rest);
    let ret = Node::from_item(op, matched_span);

    Ok((rest, ret))
}

fn parse_unary_op(input: Span) -> IResult< Node> {
    let ops = "-";
    parse_op_allowed(input, ops)
}

fn to_op(c : char) -> Result<Item, ()> {
    match c {
        '+' => Ok(Item::Add),
        '-' => Ok(Item::Sub),
        '*' => Ok(Item::Mul),
        '/' => Ok(Item::Div),
        '|' => Ok(Item::Or),
        '&' => Ok(Item::And),
        '^' => Ok(Item::Xor),
        _ => Err(())
    }
}

fn parse_op(input: Span) -> IResult< Node> {
    let ops = "+-*/|&^";
    parse_op_allowed(input, ops)
}

fn parse_op_term(input: Span) -> IResult<(Node,Node)> {
    let (rest, (op, term)) = separated_pair(parse_op, multispace0, parse_term)(input)?;
    Ok((rest,(op, term)))
}

////////////////////////////////////////////////////////////////////////////////
pub fn parse_expr(input: Span) -> IResult<Node> {
    let (rest,term) = parse_term(input)?;
    let mut vec_ret = vec![term];
    let (rest, vs) = many0(preceded(multispace0,
                                    parse_op_term
                                   ))(rest)?;

    for (o,t) in vs {
        vec_ret.push(o);
        vec_ret.push(t);
    }

    let matched_span = matched_span(input, rest);

    let node = Node::from_item(Item::Expr,matched_span).with_children(vec_ret);

    Ok(( rest, node ))
}

////////////////////////////////////////////////////////////////////////////////

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_brackets() {
        let input = "(10 + 4) + 20";
        let (rest,matched) = parse_bracketed_expr(Span::new(input)).unwrap();
        println!("{:#?}", matched);
        let matched = matched.to_string();
        assert_eq!(*rest, " + 20");
        assert_eq!("(10+4)", matched);
    }

    #[test]
    fn test_get_expr() {
        let input = "3 * 4 + %101 + -10";
        let (rest,matched) = parse_expr(Span::new(input)).unwrap();
        println!("{:#?}", matched);
        let matched = matched.to_string();

        assert_eq!(*rest, "");
        assert_eq!(matched, "3*4+5+-10");

        let input = "3 * 4 + 5 - (5 * 4)";
        let (rest,matched) = parse_expr(Span::new(input)).unwrap();
        let matched = matched.to_string();

        assert_eq!(*rest, "");
        assert_eq!(matched, "3*4+5-(5*4)");
    }
    fn test_expr_pc() {
        let input = Span::new("* ;; ");
        let (rest,matched) = parse_expr(input).unwrap();
        assert_eq!(*rest, " ;; ");
        assert_eq!(&matched.to_string(), "*");
    }
    
    fn test_parse_pc() {
        let input = Span::new("* ;; ");
        let (rest,matched) = parse_pc(input).unwrap();
        assert_eq!(*rest, " ;; ");
        assert_eq!(&matched.to_string(), "*");
    }
}
