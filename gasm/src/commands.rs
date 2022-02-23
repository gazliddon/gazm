use lazy_static::lazy_static;
use std::{collections::HashMap, path::PathBuf};

use super::{expr, util};

use Item::*;

type CommandParseFn = fn(Span) -> IResult<Node>;

use crate::{astformat::as_string, item::{Item, Node}, locate::matched_span, util::{match_escaped_str, match_file_name}};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, anychar, multispace0, multispace1},
    combinator::{map, opt, recognize},
    error::ErrorKind::NoneOf,
    multi::{many1, separated_list1},
    sequence::{preceded, separated_pair, tuple},
};

use expr::parse_expr;

use crate::error::{IResult, ParseError};
use crate::locate::Span;

fn parse_org_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Org, input).with_child(matched);
    Ok((rest, ret))
}

fn parse_fcc_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = util::wrapped_chars('"', is_not("\""), '"')(input)?;
    let ret = Node::from_item_span(Fcc(matched.to_string()), input);
    Ok((rest, ret))
}

fn parse_fdb_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = util::sep_list1(parse_expr)(input)?;
    let num_of_bytes = matched.len();
    let ret = Node::from_item_span(Fdb(num_of_bytes), input).with_children(matched);
    Ok((rest, ret))
}

fn parse_rmb_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Rmb, input).with_child(matched);
    Ok((rest, ret))
}

fn parse_fcb_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = util::sep_list1(parse_expr)(input)?;
    let num_of_bytes = matched.len();
    let ret = Node::from_item_span(Fcb(num_of_bytes), input).with_children(matched);
    Ok((rest, ret))
}

fn parse_include_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = match_escaped_str(input)?;
    let matched = matched.to_string();
    let ret = Node::from_item_span(Include(PathBuf::from(&matched)), input);
    Ok((rest, ret))
}

fn parse_incbin_arg(input: Span) -> IResult<Node> {
    use crate::util::ws;
    let sep = ws(tag(","));
    let sep2 = ws(tag(","));

    let (rest, file) = match_file_name(input)?;

    let (rest, extra_args) = opt(preceded(sep, separated_list1(sep2,parse_expr)))(rest)?;

    let ret = Node::from_item_span(IncBin(PathBuf::from(&file.to_string())), input).
        with_children(extra_args.unwrap_or(vec![]));

    Ok((rest, ret))
}

fn parse_fill_arg(input: Span) -> IResult<Node> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, matched) = separated_pair(parse_expr, sep, parse_expr)(input)?;
    let ret = mk_fill(input, matched);
    Ok((rest, ret))
}

fn parse_zmb_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Zmb, input).with_child(matched);
    Ok((rest, ret))
}

fn parse_zmd_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Zmd, input).with_child(matched);
    Ok((rest, ret))
}

fn parse_setdp_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(SetDp, input).with_child(matched);
    Ok((rest, ret))
}

fn parse_put_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Put, input).with_child(matched);
    Ok((rest, ret))
}

fn mk_fill(input: Span, cv: (Node, Node)) -> Node {
    let (count, value) = cv;
    Node::from_item_span(Fill, input).with_children(vec![count, value])
}

fn parse_bsz_arg(input: Span) -> IResult<Node> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let two_args = separated_pair(parse_expr, sep, parse_expr);
    let one_arg = map(parse_expr, |x: Node| (x, Node::from_number(0, input)));
    let (rest, matched) = alt((two_args, one_arg))(input)?;
    let ret = mk_fill(input, matched);
    Ok((rest, ret))
}

lazy_static! {
    static ref PARSE_ARG: HashMap<&'static str, CommandParseFn> = {
        let v: Vec<(_, CommandParseFn)> = vec![
            ("put", parse_put_arg),
            ("incbin", parse_incbin_arg),
            ("setdp", parse_setdp_arg),
            ("bsz", parse_bsz_arg),
            ("fill", parse_fill_arg),
            ("fdb", parse_fdb_arg),
            ("fcc", parse_fcc_arg),
            ("fcb", parse_fcb_arg),
            ("zmb", parse_zmb_arg),
            ("zmd", parse_zmd_arg),
            ("rmb", parse_rmb_arg),
            ("org", parse_org_arg),
            ("include", parse_include_arg),
        ];
        v.into_iter().collect()
    };
}

fn command_token_function(input: Span) -> IResult<(Span, CommandParseFn)> {
    use nom::error::Error;

    let (rest, matched) = alpha1(input)?;
    let token = matched.to_string().to_lowercase();

    if let Some(func) = PARSE_ARG.get(token.as_str()) {
        Ok((rest, (matched, *func)))
    } else {
        Err(crate::error::parse_error(
            "This is not a command token",
            input,
        ))
    }
}

pub fn command_token(input: Span) -> IResult<Span> {
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
