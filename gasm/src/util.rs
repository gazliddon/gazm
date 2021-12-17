use crate::item::Item;
use crate::numbers;

use nom::IResult;
use nom::branch::alt;
use std::collections::HashSet;
use nom::error::{Error, ParseError};
use nom::bytes::complete::{
    escaped, is_a, tag, tag_no_case, take_until, take_until1, take_while, take_while1,
};

use nom::character::complete::{
    alpha1, alphanumeric1, anychar, char as nom_char, line_ending, multispace0, multispace1,
    not_line_ending, one_of, satisfy, space1,
};
use nom::multi::{many0, many0_count, many1, separated_list0};
use nom::sequence::{ terminated,preceded, tuple, pair };
use nom::combinator::{ cut, eof, not, recognize };

use crate::{ opcode_token, command_token };


pub fn parse_register(_input : &str) -> IResult<&str, Item> {
    todo!()
}

pub fn parse_number(input: &str) -> IResult<&str, Item> {
    let (rest, (num, text)) = numbers::number_token(input)?;
    Ok((rest, Item::Number(num, text)))
}

static LIST_SEP: &'static str = ",";
pub fn generic_arg_list(input: &str) -> IResult<&str, Vec<&str>> {
    let sep = tuple((multispace0, tag(LIST_SEP), multispace0));
    separated_list0(sep, generic_arg)(input)
}

pub fn generic_arg(input: &str) -> IResult<&str, &str> {
    let term = alt((eof, line_ending, tag(LIST_SEP)));
    recognize(not(term))(input)
}

pub fn parse_not_sure(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = generic_arg(input)?;
    Ok((rest, Item::NotSure(matched)))
}

pub fn get_token<'a>(input: &'a str, hs: &HashSet<&'static str>) -> IResult<&'a str, &'a str> {
    use nom::error::ErrorKind::NoneOf;
    let (rest, matched) = alpha1(input)?;
    let opcode = String::from(matched).to_lowercase();

    if hs.contains(&opcode.as_str()) {
        Ok((rest, matched))
    } else {
        Err(nom::Err::Error(Error::new(input, NoneOf)))
    }
}
////////////////////////////////////////////////////////////////////////////////
// Labels
static LOCAL_LABEL_PREFIX: &'static str = "@!";
static OK_LABEL_CHARS: &'static str = "_?";

fn get_label(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = recognize(pair(
            alt((alpha1, is_a(OK_LABEL_CHARS))),
            many0(alt((alphanumeric1, is_a(OK_LABEL_CHARS)))),
            ))(input)?;

    Ok((rest, Item::Label(matched)))
}

fn get_local_label(input: &str) -> IResult<&str, Item> {
    let loc_tabs = is_a(LOCAL_LABEL_PREFIX);
    let (rest, matched) = recognize(pair(loc_tabs, get_label))(input)?;
    Ok((rest, Item::LocalLabel(matched)))
}

// pub fn alt<I: Clone, O, E: ParseError<I>, List: Alt<I, O, E>>(
//   mut l: List,
// ) -> impl FnMut(I) -> IResult<I, O, E> {
//   move |i: I| l.choice(i)
// }


pub fn parse_label(input: &str) -> IResult<&str, Item> {
    not(opcode_token)(input)?;
    not(command_token)(input)?;
    alt((get_local_label, get_label))(input)
}

////////////////////////////////////////////////////////////////////////////////
// Args

pub fn parse_arg_list(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = generic_arg_list(input)?;

    let mut ret = vec![];

    for i in matched {
        let (_, matched) = parse_arg(i)?;
        ret.push(matched);
    }

    Ok((rest, Item::ArgList(ret)))
}

pub fn parse_arg(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = alt((parse_escaped_str, parse_label, parse_not_sure))(input)?;
    Ok((rest, matched))
}

////////////////////////////////////////////////////////////////////////////////
// Escaped string

pub fn match_str(input: &str) -> IResult<&str, &str> {
    let term = "\"n\\";
    let body = take_while(move |c| !term.contains(c));
    escaped(body, '\\', one_of(term))(input)
}

pub fn match_escaped_str(input: &str) -> IResult<&str, &str> {
    preceded(nom_char('\"'), cut(terminated(match_str, nom_char('\"'))))(input)
}

pub fn parse_escaped_str(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = match_escaped_str(input)?;
    Ok((rest, Item::String(matched)))
}

mod test {
    use super::*;

    #[test]
    fn test_parse_str() {
        let res = parse_escaped_str("\"kjskjbb\"");
        println!("res : {:?}", res);
        assert!(res.is_ok())
    }

}


