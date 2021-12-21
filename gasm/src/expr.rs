// Parse expressions
//
use nom::IResult;
use super::item::Item;
use nom::error::Error;
use nom::error::ErrorKind::NoneOf;
use nom::character::complete::{multispace0, char as nom_char };

use nom::bytes::complete::{
    is_a, tag, 
};

use nom::sequence::terminated;

use nom::branch::alt;

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

pub fn parse_bracket(input: &str) -> IResult<&str, Item> {
    let (rest, matched)= alt(( nom_char('('), nom_char(')') ))(input)?;

    let ret = match matched {
        '(' => Item::OpenBracket,
        ')' => Item::CloseBracket,
        _ => panic!("something has gone wrong")
    };

    Ok((rest, ret))
}

pub fn parse_op(input: &str) -> IResult<&str, Item> {
    let double_ops = alt(( tag("++"), tag("--") ));
    let single_ops = is_a("+-*/");

    let (rest, matched)= alt((double_ops, single_ops))(input)?;

    Ok((rest, Item::Op(matched.to_string())))
}

pub fn expr_item(input : &str) -> IResult<&str, Item> {
    let (rest, matched) = alt(
        ( parse_label,
          util::parse_number,
          parse_bracket,
          parse_op)
        )(input)?;
    Ok((rest, matched))
}

pub fn parse_expr(input: &str) -> IResult<&str, Item> {
    let mut items = vec![];

    let mut input = input;

    while let Ok((rest, matched)) = terminated(expr_item, multispace0)(input) {
            items.push(matched);
            input = rest;
    }

    if items.is_empty() {
        Err(nom::Err::Error(Error::new(input, NoneOf)))
    } else {
        Ok((input, Item::Expr(items)))
    }
}

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
