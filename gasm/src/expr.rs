
// Parse expressions
//
use nom::IResult;
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

fn parse_bracketed_expr(input: &str) -> IResult<&str, Node> {
    util::wrapped_chars('(', parse_expr, ')')(input)
}

fn parse_pc(input : &str) -> IResult<&str, Node> {
    let (rest, _matched) = nom_char('*')(input)?;
    Ok((rest, Node::from_item(Item::Pc)))
}

fn parse_non_unary_term(input: &str) -> IResult<&str, Node> {
    use util::parse_number;

    alt((parse_bracketed_expr,
          parse_number,
          parse_label,
          parse_pc,
          ))(input)
}

pub fn parse_term(input: &str) -> IResult<&str, Node> {
    alt((parse_unary_term, parse_non_unary_term))(input)
}

fn parse_unary_term(input: &str) -> IResult<&str, Node> {
    use util::parse_number;
    let (rest, (op, term)) = separated_pair(parse_unary_op,  multispace0, parse_term)(input)?;
    let ret = Node::from_item(Item::UnaryTerm).with_children(vec![op,term]);
    Ok((rest, ret))
}

fn parse_unary_op(input: &str) -> IResult<&str, Node> {
    let ops = "+-";

    let (rest, matched) = one_of(ops)(input)?;

    let op = match matched {
        '+' => Item::UnaryPlus,
        '-' => Item::UnaryMinus,
        _ => panic!("{:?}", matched),
    };

    let ret = Node::from_item(op);
    Ok((rest, ret))
}

fn parse_op(input: &str) -> IResult<&str, Node> {
    let ops = "+-*/";

    let (rest, matched) = one_of(ops)(input)?;

    let op = match matched {
        '+' => Item::Add,
        '-' => Item::Sub,
        '*' => Item::Mul,
        '/' => Item::Div,
        _ => panic!("{:?}", matched),
    };
    let ret = Node::from_item(op);

    Ok((rest, ret))
}

fn parse_op_term(input: &str) -> IResult<&str, Node> {
    let (rest, (op, term)) = separated_pair(parse_op, multispace0, parse_term)(input)?;
    let node = op.with_child(term.into());
    Ok((rest,node))
}

fn prepend(i : Node, is : Vec<Node>) -> Vec<Node> {
    let mut ret = vec![i];
    ret.extend(is);
    ret
}

////////////////////////////////////////////////////////////////////////////////
pub fn parse_expr(input: &str) -> IResult<&str, Node> {
    let (rest, (v,vs)) = separated_pair(parse_term, multispace0, many0(parse_op_term))(input)?;

    if vs.is_empty() {
        Ok((rest,v))
    } else {
        let v = prepend(v,vs);
        let node = Node::from_item(Item::Expr).with_children(v);
        Ok((rest,node))
    }

}

////////////////////////////////////////////////////////////////////////////////

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    fn mk_res_op(input : &str) -> IResult<&str, Item> {
        Ok(("", Item::Op(input.to_string())))
    }

    #[test]
    fn test_op() {
        let res = parse_op("++");
        assert_eq!(res, mk_res_op("++"));

        let res = parse_op("--");
        assert_eq!(res, mk_res_op("--"));

        let res = parse_op("+");
        assert_eq!(res, mk_res_op("+"));

        let res = parse_op("-");
        assert_eq!(res, mk_res_op("-"));

        let res = parse_op("!");
        assert!( res.is_err() );
    }

    #[test]
    fn test_expr_item() {
        let res = expr_item("hello");
        assert_eq!(res, Ok(("", Item::Label("hello".to_string()))));

        let res = expr_item("!hello");
        assert_eq!(res, Ok(("", Item::LocalLabel("!hello".to_string()))));

        let res = expr_item("0xffff");
        assert_eq!(res, Ok(("", Item::Number(65535))));

        let res = expr_item("()");
        assert_eq!(res, Ok((")", Item::OpenBracket)));
        let res = expr_item(")");
        assert_eq!(res, Ok(("", Item::CloseBracket)));

        let res = expr_item("-");
        assert_eq!(res, mk_res_op("-"));
    }

    #[test]
    fn test_get_expr() {

        let desired =Item::Expr(vec![
                                Item::Label("hello".to_string()), 
                                Item::Op("+".to_string()),
                                Item::Number(4096),
        ]);

        let res = parse_expr("hello + $1000");
        assert_eq!(res,Ok(("", desired.clone())));

        let res = parse_expr("hello+ $1000");
        assert_eq!(res,Ok(("", desired.clone())));

        let res = parse_expr("hello+ $1000!!!!");
        assert_eq!(res,Ok(("!!!!", desired.clone())));

        let desired =Item::Expr(vec![
                                Item::LocalLabel("!hello".to_string()), 
                                Item::Op("+".to_string()),
                                Item::Number(4096),
        ]);

        let res = parse_expr("!hello+ $1000!!!!");
        assert_eq!(res,Ok(("!!!!", desired.clone())));
        let res = parse_expr("!hello+ $1000!!!!");
        assert_eq!(res,Ok(("!!!!", desired.clone())));
    }
}
