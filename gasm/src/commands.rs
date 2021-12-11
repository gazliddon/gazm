use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};


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

    static ref PARSE_ARG: HashMap<&'static str, for <'a> fn(&'a str, &'a str) -> IResult<&'a str, Command<'a>>> = {
        let mut hs = HashMap::<&'static str, for <'a> fn(&'a str, &'a str) -> IResult<&'a str, Command<'a>>>::new();

        hs.insert("fdb", parse_command_arg);
        hs.insert("org", parse_command_arg);
        hs.insert("include", parse_include_arg);
        hs
    };
}

// pub type IResult<I, O, E = error::Error<I>> = Result<(I, O), Err<E>>;

pub fn parse_command<'a>(input: &'a str) -> IResult<&'a str,Item> {

    let mapper = |matched : &'a str| -> IResult<&'a str, for <'x> fn(&'x str, &'x str)-> IResult<&'x str, Command<'x>>> {
        let e = nom::error::Error{input, code: nom::error::ErrorKind::IsNot};
        if let Some(func) = PARSE_ARG.get(matched) {
            Ok(( matched,*func ))
        } else {
            Err(nom::Err::Error(e))
        }
    };

    let (rest, (command, func)) = map_res(alpha1,mapper)(input)?;
    let (rest, matched) = preceded(multispace1, |input| func(command, input))(rest)?;
    let i = Item::CommandWithArg(matched);
    Ok((rest, i))
}

mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
}
