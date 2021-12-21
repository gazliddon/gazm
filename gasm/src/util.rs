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
// Args
pub fn generic_arg_list(input: &str) -> IResult<&str, Vec<&str>> {
    let sep = tuple((multispace0, tag(LIST_SEP), multispace0));
    separated_list0(sep, generic_arg)(input)
}

pub fn generic_arg(input: &str) -> IResult<&str, &str> {
    let term = alt((eof, line_ending, tag(LIST_SEP)));
    recognize(not(term))(input)
}

////////////////////////////////////////////////////////////////////////////////
// Labels
static LOCAL_LABEL_PREFIX: &str = "@!";
static OK_LABEL_CHARS: &str = "_?.";

fn get_label_identifier(input: &str) -> IResult<&str, &str> {
    // match a label identifier
    let (rest,matched) = recognize(pair(
            alt((alpha1, is_a(OK_LABEL_CHARS))),
            many0(alt((alphanumeric1, is_a(OK_LABEL_CHARS)))),
            ))(input)?;

    // opcodes and commands are reserved
    not( alt((opcode_token, command_token))
       )(matched)?;

    Ok((rest, matched))
}

fn get_label(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = get_label_identifier(input)?;
    Ok((rest, Item::Label(matched.to_string())))
}

fn get_local_label(input: &str) -> IResult<&str, Item> {
    let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    let prefix_parse = recognize(pair(loc_tags, get_label_identifier));

    let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    let postfix_parse = recognize(pair( get_label_identifier, loc_tags));

    let (rest, matched) = alt((postfix_parse, prefix_parse))(input)?;
    Ok((rest, Item::LocalLabel(matched.to_string())))
}

pub fn parse_label(input: &str) -> IResult<&str, Item> {
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
    let (rest, matched) = alt((parse_escaped_str, parse_label))(input)?;
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

    #[test]
    fn test_parse_label() {
        let nl = "non_local".to_string();
        let res = parse_label(&nl);
        assert_eq!(res, Ok(("", Item::Label(nl.clone()))));
        let res = parse_label("adc");
        assert_eq!(res, Ok(("", Item::Label("adc".to_string()))));
    }
    fn mk_label(a : &str) -> Item {
        Item::Label(a.to_string())
    }
    fn mk_loc_label(a : &str) -> Item {
        Item::LocalLabel(a.to_string())
    }

    #[test]
    fn test_parse_local_label() {
        let lab_str = "@_local";
        let res = parse_label(lab_str);
        let des = mk_loc_label(lab_str);
        assert_eq!(res, Ok(("", des)));


        let lab_str = "local@";
        let res = parse_label(lab_str);
        let des = mk_loc_label(lab_str);
        assert_eq!(res, Ok(("", des)));

        let lab_str = "local!";
        let res = parse_label(lab_str);
        let des = mk_loc_label(lab_str);
        assert_eq!(res, Ok(("", des)));

        let lab_str = "!local_6502";
        let res = parse_label(lab_str);
        let des = mk_loc_label(lab_str);
        assert_eq!(res, Ok(("", des)));
    }

    #[test]
    fn test_label_no_opcodes() {
        let res = parse_label("NEG");
        assert_ne!(res, Ok(("",  Item::Label("NEG".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("neg");
        assert_ne!(res, Ok(("",  Item::Label("neg".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("negative");
        assert_eq!(res, Ok(("",  Item::Label("negative".to_string()) )) );
    }

    #[test]
    fn test_label_no_commands() {
        let res = parse_label("fdb");
        assert_ne!(res, Ok(("",  Item::Label("fdb".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("org");
        assert_ne!(res, Ok(("",  Item::Label("org".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("!org");
        assert_ne!(res, Ok(("",  Item::LocalLabel("org".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("equation");
        assert_eq!(res, Ok(("",  Item::Label("equation".to_string()) )) );
    }
}


