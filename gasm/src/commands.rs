use lazy_static::lazy_static;
use std::{collections::{HashMap, HashSet}, os::unix::prelude::CommandExt};

use super::{ expr, util };

type CommandParseFn = for <'x> fn(&'x str, &'x str)-> IResult<&'x str, Command<'x>>;

use crate::{
    item::{Item, Command},
    util::match_escaped_str,
};

use nom::{
    error::{Error, ErrorKind::NoneOf, },
    character::complete::{anychar, multispace0,multispace1, alpha1},
    combinator::{cut, recognize, map_res},
    multi::{separated_list1, many1 },
    sequence::{ separated_pair, preceded, tuple, },
    bytes::complete::tag,
    IResult,
};

fn parse_command_arg<'a>(command : &'a str, input: &'a str) -> IResult<&'a str, Command<'a>> {
    let (rest, matched) = recognize(many1(anychar))(input)?;
    Ok((rest, Command::Generic(command, Some(matched))))
}

fn parse_org_arg<'a>(_command : &'a str, input: &'a str) -> IResult<&'a str, Command<'a>> {
    let (rest, matched) = expr::parse_expr(input)?;
    Ok((rest, Command::Org(Box::new(matched))))
}
fn parse_fdb_arg<'a>(_command : &'a str, input: &'a str) -> IResult<&'a str, Command<'a>> {
    let (rest, matched) = util::sep_list1(expr::parse_expr)(input)?;
    Ok((rest, Command::Fdb(matched)))
}

fn parse_include_arg<'a>(_command: &'a str, input : &'a str) -> IResult<&'a str, Command<'a>> {
    let (rest, matched) = match_escaped_str(input)?;
    Ok((rest, Command::Include(matched)))
}

fn parse_generic_arg<'a>(command: &'a str, input : &'a str) -> IResult<&'a str, Command<'a>> {
    let (rest, matched) = recognize(many1(anychar))(input)?;
    Ok((rest, Command::Generic(command,Some(matched))))
}

fn parse_fill_arg<'a>(_command : &'a str, input: &'a str) -> IResult<&'a str, Command<'a>> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, (amount, value)) = separated_pair(expr::parse_expr, sep, expr::parse_expr)(input)?;
    Ok((rest, Command::Fill(
                Box::new( amount ),
                Box::new( value )
                )))
}

fn parse_fill_zero_arg<'a>(_command: &'a str, input : &'a str) -> IResult<&'a str, Command<'a>> {
    let (rest, matched) = expr::parse_expr(input)?;
    Ok((rest, Command::FillZero(Box::new(matched))))
}

lazy_static! {
    static ref PARSE_ARG: HashMap<&'static str, CommandParseFn>= {
        let mut hs = HashMap::<&'static str, CommandParseFn>::new();

        hs.insert("bsz", parse_fill_zero_arg);
        hs.insert("fill", parse_fill_arg);
        hs.insert("fdb", parse_fdb_arg);
        hs.insert("rmb", parse_fdb_arg);
        hs.insert("org", parse_org_arg);
        hs.insert("include", parse_include_arg);
        hs.insert("setdp", parse_generic_arg);
        hs
    };
}

fn command_token_function<'a>(input: &'a str) -> IResult<&'a str,(&'a str, CommandParseFn )> {
    use nom::error::{Error, ParseError};

    let (rest, matched) = alpha1(input)?;
    let token = String::from(matched).to_lowercase();

    if let Some(func) = PARSE_ARG.get(token.as_str()) {
        Ok((rest, (matched, *func )))
    } else {
        Err(nom::Err::Error(Error::new(input, NoneOf)))
    }
}

pub fn command_token(input: &str) -> IResult<&str, &str> {
    let (rest, (matched, _)) = command_token_function(input)?;
    Ok((rest, matched))
}

pub fn parse_command<'a>(input: &'a str) -> IResult<&'a str,Item> {
    let (rest, (command_text, func )) = command_token_function(input)?;
    let (rest, matched) = preceded(multispace1, |input| func(command_text, input))(rest)?;
    let i = Item::Command(matched);
    Ok((rest, i))
}

mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
}
