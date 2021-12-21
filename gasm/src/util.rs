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
use nom::multi::{many0, many0_count, many1, separated_list1,separated_list0};
use nom::sequence::{ delimited, terminated,preceded, tuple, pair };
use nom::combinator::{ cut, eof, not, recognize };

use crate::{ opcode_token, command_token };
pub static LIST_SEP: & str = ",";


pub fn ws<'a, F, O, E>( mut inner: F,) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
  F: nom::Parser<&'a str, O, E> + 'a,
  E: ParseError<& 'a str>,
{
  move |input: &'a str| {
    let (input, _) = multispace0(input)?;
    let (input, out) = inner.parse(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input,out))
  }
}

pub fn wrapped<'a, O1, OUT, O3, E, F, INNER, S>(
  mut first: F,
  mut inner: INNER,
  mut second: S,
) -> impl FnMut(&'a str) -> IResult<&'a str, OUT, E>
where
  E: ParseError<& 'a str>,
  F: nom::Parser<&'a str, O1, E> + 'a,
  INNER: nom::Parser<&'a str, OUT, E> + 'a,
  S: nom::Parser<&'a str, O3, E> + 'a,
{
  move |input: &'a str| {
    let (input, _) = first.parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, out) = inner.parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = second.parse(input)?;
    Ok((input,out))
  }
}

pub fn wrapped_chars<'a, O2,  E: ParseError<&'a str> + 'a, G>(
    first : char, yes_please: G, last: char)
-> impl FnMut(&'a str) -> IResult<&'a str, O2, E>
where
G: nom::Parser<&'a str, O2, E> + 'a {
    wrapped(nom_char(first),yes_please, nom_char(last))
}

pub fn sep_list1<'a, F, O, E: ParseError<&'a str>>(
    inner: F
    ) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<O>, E>
where
F: nom::Parser<&'a str, O,E>  + Copy {
    move |input: &'a str| {
        let sep = tuple((multispace0, tag(LIST_SEP), multispace0));
        separated_list1(sep, inner)(input)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Number
pub fn parse_number(input: &str) -> IResult<&str, Item> {
    let (rest, (num, _text)) = numbers::number_token(input)?;
    Ok((rest, Item::Number(num)))
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
    Ok((rest, Item::QuotedString(matched.to_string())))
}

////////////////////////////////////////////////////////////////////////////////
// Tests
mod test {
    use crate::commands::parse_command;

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_parse_str() {
        let res = parse_escaped_str("\"kjskjbb\"");
        println!("res : {:?}", res);
        assert!(res.is_ok())
    }

}


