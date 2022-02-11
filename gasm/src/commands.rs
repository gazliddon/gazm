use lazy_static::lazy_static;
use std::{collections::HashMap, path::PathBuf};

use super::{ expr, util };

use Item::*;

type CommandParseFn = fn( Span)-> IResult<Node>;

use crate::{item::{Item, Node}, locate::matched_span, util::match_escaped_str};

use nom::{
    branch::alt,
    bytes::complete::{ tag, is_not },
    character::complete::{anychar, multispace0,multispace1, alpha1},
    combinator::{recognize, map},
    error::ErrorKind::NoneOf,
    multi::many1,
    sequence::{ separated_pair, preceded, tuple, }

};

use expr::parse_expr;

use crate::error::{IResult, ParseError};
use crate::locate::Span;

fn parse_org_arg(input: Span) -> IResult< Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Org,input).with_child(matched);
    Ok((rest, ret))
}

fn parse_fcc_arg(input: Span) -> IResult< Node> {
    let (rest, matched) = recognize(util::wrapped_chars('\'',is_not("'"), '\''))(input)?;
    let ret = Node::from_item_span(Fcc(matched.to_string()), input);
    Ok((rest, ret))
}

fn parse_fdb_arg(input: Span) -> IResult< Node> {
    let (rest, matched) = util::sep_list1(parse_expr)(input)?;
    let num_of_bytes = matched.len();
    let ret = Node::from_item_span(Fdb(num_of_bytes), input).with_children(matched);
    Ok((rest, ret))
}

fn parse_fcb_arg(input: Span) -> IResult< Node> {
    let (rest, matched) = util::sep_list1(parse_expr)(input)?;
    let num_of_bytes = matched.len();
    let ret = Node::from_item_span(Fcb(num_of_bytes), input).with_children(matched);
    Ok((rest, ret))
}


fn parse_include_arg(input : Span) -> IResult< Node> {
    let (rest, matched) = match_escaped_str(input)?;
    let matched = matched.to_string();
    let ret = Node::from_item_span(Include(PathBuf::from(&matched)), input);
    Ok((rest, ret))
}

fn parse_set_dp(input : Span) -> IResult< Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(SetDp, input).with_child(matched);
    Ok((rest,ret)) 
}

fn parse_fill_arg( input: Span) -> IResult< Node> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, matched) = separated_pair(parse_expr, sep, parse_expr)(input)?;
    let ret = mk_fill(input, matched);
    Ok((rest, ret))
}

fn parse_zmb_arg( input: Span) -> IResult< Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Zmb, input).with_child(matched);
    Ok((rest, ret))
}

fn parse_zmd_arg( input: Span) -> IResult< Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Zmd, input).with_child(matched);
    Ok((rest,ret))
}

fn mk_fill(input : Span, cv: ( Node, Node) ) -> Node {
    let (count, value) = cv;
    Node::from_item_span(Fill, input).with_children(vec![count,value])
}

fn parse_bsz_arg( input : Span) -> IResult< Node> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let two_args = separated_pair(parse_expr, sep, parse_expr);
    let one_arg = map(parse_expr, |x : Node| (x,Node::from_number(0, input)));
    let (rest, matched) = alt(( two_args, one_arg))(input)?;
    let ret = mk_fill(input, matched);
    Ok((rest, ret))
}

lazy_static! {
    static ref PARSE_ARG: HashMap<&'static str, CommandParseFn>= {
        let v : Vec<(_, CommandParseFn)>= vec![
            ("bsz", parse_bsz_arg),
            ("fill", parse_fill_arg),
            ("fdb", parse_fdb_arg),
            ("fcc", parse_fcc_arg),
            ("fcb", parse_fcb_arg),
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

fn command_token_function(input: Span) -> IResult< (Span, CommandParseFn ) > {
    use nom::error::Error;

    let (rest, matched) = alpha1(input)?;
    let token = matched.to_string().to_lowercase();

    if let Some(func) = PARSE_ARG.get(token.as_str()) {
        Ok((rest, (matched, *func )))
    } else {
        Err(
            crate::error::parse_error("This is not a command token", input))
    }
}

pub fn command_token(input: Span) -> IResult< Span> {
    let (rest, (matched, _)) = command_token_function(input)?;
    Ok((rest, matched))
}

pub fn parse_command(input: Span) -> IResult<Node> {
    let (rest, (_command_text, func)) = command_token_function(input)?;
    let (rest, matched) = preceded(multispace1, func)(rest)?;
    let span = matched_span(input, rest);
    let matched = matched.with_span(span);
    Ok((rest, matched))
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
}
