use crate::item::Item;
use crate::numbers;

use nom::IResult;
use nom::error::ParseError;
use nom::bytes::complete::{
    escaped,
    tag,
    take_while,
};

use nom::character::complete::{
    char as nom_char, multispace0,
    one_of, 
};
use nom::multi::separated_list1;
use nom::sequence::{ terminated,preceded, tuple, };
use nom::combinator::cut;

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
// Escaped string

pub fn match_str(input: &str) -> IResult<&str, &str> {
    let term = "\"n\\";
    let body = take_while(move |c| !term.contains(c));
    escaped(body, '\\', one_of(term))(input)
}

pub fn match_escaped_str(input: &str) -> IResult<&str, &str> {
    preceded(nom_char('\"'), cut(terminated(match_str, nom_char('\"'))))(input)
}

////////////////////////////////////////////////////////////////////////////////
// Number

pub fn parse_escaped_str(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = match_escaped_str(input)?;
    Ok((rest, Item::QuotedString(matched.to_string())))
}
pub fn parse_number(input: &str) -> IResult<&str, Item> {
    let (rest, (num, _text)) = numbers::number_token(input)?;
    Ok((rest, Item::Number(num)))
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_parse_str() {
        let res = parse_escaped_str("\"kjskjbb\"");
        println!("res : {:?}", res);
        assert!(res.is_ok())
    }

}


