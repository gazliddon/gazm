// Parse expressions
//
use nom::IResult;
use crate::{ opcode_token, command_token };
use super::numbers;
use super::item::Item;
use nom::error::{Error, ParseError};
    use nom::error::ErrorKind::NoneOf;
use nom::character::complete::{
    alpha1, alphanumeric1, anychar, char as nom_char, line_ending, multispace0, multispace1,
    not_line_ending, one_of, satisfy, space1,
};

use nom::bytes::complete::{
    escaped, is_a, tag, tag_no_case, take_until, take_until1, take_while, take_while1,
};

use nom::branch::alt;

use super::util;

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

    Ok((rest, Item::Op(matched)))
}

pub fn expr_item(input : &str) -> IResult<&str, Item> {
    let (rest, matched) = alt(
        ( util::parse_label,
          util::parse_number,
          parse_bracket,
          parse_op)
        )(input)?;
    Ok((rest, matched))
}

pub fn get_expr(input: &str) -> IResult<&str, Item> {

    let mut items = vec![];

    let mut input = input;

    loop {
        if let Ok((rest, matched)) = expr_item(input) {
            items.push(matched);
            input = rest;
        } else {
            break;
        }
    }

    if items.is_empty() {
        Err(nom::Err::Error(Error::new(input, NoneOf)))
    } else {
        Ok((input, Item::Expr(items)))
    }
}
