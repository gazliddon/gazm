use lazy_static::lazy_static;
use std::{collections::HashMap, path::PathBuf};

use super::{ expr, util };

use Item::*;

type CommandParseFn = fn( &str)-> IResult<&str, Node>;

use crate::{
    item::{Item, Node},
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

fn parse_org_arg(input: & str) -> IResult<& str, Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item(Org).with_child(matched);
    Ok((rest, ret))
}

fn parse_fdb_arg(input: &str) -> IResult<&str, Node> {
    let (rest, matched) = util::sep_list1(parse_expr)(input)?;
    let ret = Node::from_item(Fdb).with_children(matched);
    Ok((rest, ret))
}

fn parse_include_arg(input : & str) -> IResult<&str, Node> {
    let (rest, matched) = match_escaped_str(input)?;
    let ret = Node::from_item(Include(PathBuf::from(matched)));
    Ok((rest, ret))
}

fn parse_set_dp(input : &str) -> IResult<&str, Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item(SetDp).with_child(matched);
    Ok((rest,ret)) 
}

fn parse_fill_arg( input: &str) -> IResult<&str, Node> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    map(separated_pair(parse_expr, sep, parse_expr), mk_fill)(input)
}

fn parse_zmb_arg( input: &str) -> IResult<&str, Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item(Zmb).with_child(matched);
    Ok((rest, ret))
}

fn parse_zmd_arg( input: &str) -> IResult<&str, Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item(Zmd).with_child(matched);
    Ok((rest,ret))
}

fn mk_fill(cv: ( Node, Node) ) -> Node {
    let (count, value) = cv;
    Node::from_item(Fill).with_children(vec![count,value])
}

fn parse_bsz_arg( input : &str) -> IResult<&str, Node> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let two_args = separated_pair(parse_expr, sep, parse_expr);
    let one_arg = map(parse_expr, |x : Node| (x,Node::from_number(0)));
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
            ("setdp", parse_set_dp),
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

pub fn parse_command(input: &str) -> IResult<&str,Node> {
    let (rest, (_command_text, func )) = command_token_function(input)?;
    let (rest, matched) = preceded(multispace1,  func)(rest)?;
    Ok((rest, matched))
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
}
