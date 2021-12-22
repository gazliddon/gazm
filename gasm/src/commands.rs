use lazy_static::lazy_static;
use std::{collections::HashMap, path::PathBuf};

use super::{ expr, util };

type CommandParseFn = for <'x> fn(&'x str, &'x str)-> IResult<&'x str, Command>;

use crate::{
    item::{Item, Command},
    util::match_escaped_str
};

use nom::{
    IResult, 
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, multispace0,multispace1, alpha1},
    combinator::{recognize, map},
    error::ErrorKind::NoneOf,
    multi::many1,
    sequence::{ separated_pair, preceded, tuple, }
};

use expr::parse_expr;

fn parse_org_arg<'a>(_command : &'a str, input: &'a str) -> IResult<&'a str, Command> {
    let (rest, matched) = parse_expr(input)?;
    Ok((rest, Command::Org(Box::new(matched))))
}

fn parse_fdb_arg<'a>(_command : &'a str, input: &'a str) -> IResult<&'a str, Command> {
    let (rest, matched) = util::sep_list1(parse_expr)(input)?;
    Ok((rest, Command::Fdb(matched)))
}

fn parse_include_arg<'a>(_command: &'a str, input : &'a str) -> IResult<&'a str, Command> {
    let (rest, matched) = match_escaped_str(input)?;
    Ok((rest, Command::Include(PathBuf::from(matched))))
}

fn parse_generic_arg<'a>(command: &'a str, input : &'a str) -> IResult<&'a str, Command> {
    let (rest, matched) = recognize(many1(anychar))(input)?;
    Ok((rest, Command::Generic(command.to_string(),Some(matched.to_string()))))
}

fn parse_fill_arg<'a>(_command : &'a str, input: &'a str) -> IResult<&'a str, Command> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    map(separated_pair(parse_expr, sep, parse_expr), mk_fill)(input)
}

fn get_box_expr<'a>(input: &'a str) -> IResult<&'a str, Box<Item>> {
    let (rest, matched) = parse_expr(input).map(|(r, m )| (r, Box::new(m)))?;
    Ok((rest, matched))
}


fn parse_zmb_arg<'a>(_command : &'a str, input: &'a str) -> IResult<&'a str, Command> {
    let (rest, matched) = get_box_expr(input)?;
    Ok((rest,Command::Zmb(matched)))
}

fn parse_zmd_arg<'a>(_command : &'a str, input: &'a str) -> IResult<&'a str, Command> {
    let (rest, matched) = get_box_expr(input)?;
    Ok((rest,Command::Zmb(matched)))
}

fn mk_fill(cv: ( Item, Item) ) -> Command {
    let (count, value) = cv;
    Command::Fill(Box::new(value), Box::new(count))
}

fn parse_bsz_arg<'a>(_command: &'a str, input : &'a str) -> IResult<&'a str, Command> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let two_args = separated_pair(parse_expr, sep, parse_expr);
    let one_arg = map(parse_expr, |x : Item| (x,Item::Number(0)));

    map(alt(( two_args, one_arg)), mk_fill)(input)
}

lazy_static! {
    static ref PARSE_ARG: HashMap<&'static str, CommandParseFn>= {
        let v : Vec<(_, CommandParseFn)>= vec![
            ("bsz", parse_bsz_arg),
            ("fill", parse_fill_arg),
            ("fdb", parse_fdb_arg),
            ("zmb", parse_zmb_arg),
            ("zmd", parse_zmd_arg),
            ("rmb", parse_fdb_arg),
            ("org", parse_org_arg),
            ("include", parse_include_arg),
            ("setdp", parse_generic_arg),
        ];

        v.into_iter().collect()
    };
}

fn command_token_function(input: &str) -> IResult<&str,(&str, CommandParseFn )> {
    use nom::error::Error;

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

pub fn parse_command(input: &str) -> IResult<&str,Item> {
    let (rest, (command_text, func )) = command_token_function(input)?;
    let (rest, matched) = preceded(multispace1, |input| func(command_text, input))(rest)?;
    let i = Item::Command(matched);
    Ok((rest, i))
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
}
