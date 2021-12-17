use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

type CommandParseFn = for <'x> fn(&'x str, &'x str)-> IResult<&'x str, Command<'x>>;

use crate::{
    item::{Item, Command},
    util::{get_token, match_escaped_str},
};

use nom::{
    error::ErrorKind::NoneOf,
    error::Error,
    character::complete::{anychar, multispace1, alpha1},
    combinator::{cut, recognize, map_res},
    multi::many1,
    sequence::{ separated_pair, preceded },
    IResult,
};

fn parse_command_arg<'a>(command : &'a str, input: &'a str) -> IResult<&'a str, Command<'a>> {
    let (rest, matched) = recognize(many1(anychar))(input)?;
    Ok((rest, Command::Generic(command, Some(matched))))
}

fn parse_include_arg<'a>(_command: &'a str, input : &'a str) -> IResult<&'a str, Command<'a>> {
    let (rest, matched) = match_escaped_str(input)?;
    Ok((rest, Command::Include(matched)))
}

fn parse_generic_arg<'a>(command: &'a str, input : &'a str) -> IResult<&'a str, Command<'a>> {
    let (rest, matched) = recognize(many1(anychar))(input)?;
    Ok((rest, Command::Generic(command,Some(matched))))
}

lazy_static! {

    static ref PARSE_ARG: HashMap<&'static str, CommandParseFn>= {
        let mut hs = HashMap::<&'static str, CommandParseFn>::new();

        hs.insert("fdb", parse_command_arg);
        hs.insert("org", parse_command_arg);
        hs.insert("include", parse_include_arg);
        hs
    };
}

// pub type IResult<I, O, E = error::Error<I>> = Result<(I, O), Err<E>>;
pub fn command_token<'a>(input: &'a str) -> IResult<&'a str,(&'a str, CommandParseFn )> {
    use nom::error::{Error, ParseError};

    let (rest, matched) = alpha1(input)?;
    let opcode = String::from(matched).to_lowercase();

    if let Some(func) = PARSE_ARG.get(opcode.as_str()) {
        Ok((rest, (matched, *func )))
    } else {
        Err(nom::Err::Error(Error::new(input, NoneOf)))
    }
}

pub fn parse_command<'a>(input: &'a str) -> IResult<&'a str,Item> {
    let (rest, (command_text, func )) = command_token(input)?;
    let (rest, matched) = preceded(multispace1, |input| func(command_text, input))(rest)?;
    let i = Item::Command(matched);
    Ok((rest, i))
}

mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
}