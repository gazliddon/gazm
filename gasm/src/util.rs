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

////////////////////////////////////////////////////////////////////////////////
// Labels
static LOCAL_LABEL_PREFIX: &'static str = "@!";
static OK_LABEL_CHARS: &'static str = "_?";

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
    Ok((rest, Item::Label(matched)))
}

fn get_local_label(input: &str) -> IResult<&str, Item> {
    let loc_tabs = is_a(LOCAL_LABEL_PREFIX);
    let (rest, (_,matched)) = pair(loc_tabs, get_label_identifier)(input)?;
    Ok((rest, Item::LocalLabel(matched)))
}

// pub fn alt<I: Clone, O, E: ParseError<I>, List: Alt<I, O, E>>(
//   mut l: List,
// ) -> impl FnMut(I) -> IResult<I, O, E> {
//   move |i: I| l.choice(i)
// }


pub fn parse_label(input: &str) -> IResult<&str, Item> {
    // not(opcode_token)(input)?;
    // not(command_token)(input)?;
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
        let res = parse_label("non_local");
        assert_eq!(res, Ok(("", Item::Label("non_local"))));
        let res = parse_label("adc");
        assert_eq!(res, Ok(("", Item::Label("adc"))));
    }

    #[test]
    fn test_parse_local_label() {
        let res = parse_label("@_local");
        assert_eq!(res, Ok(("", Item::LocalLabel("_local"))));
        let res = parse_label("!local_6502");
        assert_eq!(res, Ok(("", Item::LocalLabel("local_6502"))));
    }

    #[test]
    fn test_label_no_opcodes() {
        let res = parse_label("NEG");
        assert_ne!(res, Ok(("",  Item::Label("NEG") )) );
        assert!(res.is_err());

        let res = parse_label("neg");
        assert_ne!(res, Ok(("",  Item::Label("neg") )) );
        assert!(res.is_err());

        let res = parse_label("negative");
        assert_eq!(res, Ok(("",  Item::Label("negative") )) );
    }

    #[test]
    fn test_label_no_commands() {
        let res = parse_label("fdb");
        assert_ne!(res, Ok(("",  Item::Label("fdb") )) );
        assert!(res.is_err());

        let res = parse_label("org");
        assert_ne!(res, Ok(("",  Item::Label("org") )) );
        assert!(res.is_err());

        let res = parse_label("!org");
        assert_ne!(res, Ok(("",  Item::LocalLabel("org") )) );
        assert!(res.is_err());

        let res = parse_label("equation");
        assert_eq!(res, Ok(("",  Item::Label("equation") )) );
    }
}


