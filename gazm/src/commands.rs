use lazy_static::lazy_static;
use std::{collections::HashMap, path::PathBuf};

use super::expr;

use Item::*;

type CommandParseFn = fn(Span) -> IResult<Node>;

use crate::{
    item::{Item, Node},
    labels::get_just_label,
    locate::{matched_span, span_to_pos},
    parse::util::{self,match_escaped_str, match_file_name},
};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, multispace0, multispace1},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{preceded, separated_pair, tuple},
};

use expr::parse_expr;

use crate::error::IResult;
use crate::locate::Span;

fn parse_org_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Org, input).with_child(matched);
    Ok((rest, ret))
}

fn parse_exec_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = parse_expr(input)?;
    let ret = Node::from_item_span(Exec, input).with_child(matched);
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
    let ret = Node::new_with_children(Fdb(num_of_bytes), matched, span_to_pos(input));
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
    let ret = Node::new_with_children(Fcb(num_of_bytes), matched, span_to_pos(input));
    Ok((rest, ret))
}

fn parse_include_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = match_escaped_str(input)?;
    let matched = matched.to_string();
    let ret = Node::from_item_span(Include(PathBuf::from(&matched)), input);
    Ok((rest, ret))
}

fn parse_require_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = match_escaped_str(input)?;
    let matched = matched.to_string();
    let ret = Node::from_item_span(Require(PathBuf::from(&matched)), input);
    Ok((rest, ret))
}

fn parse_grab_mem(input: Span) -> IResult<Node> {
    use util::ws;
    let (rest, (src, size)) = separated_pair(parse_expr, ws(tag(",")), parse_expr)(input)?;
    let ret = Node::new_with_children(GrabMem, vec![src, size],span_to_pos( input ));
    Ok((rest, ret))
}

fn inc_bin_args(input: Span) -> IResult<(PathBuf, Option<Vec<Node>>)> {
    use util::ws;
    let sep = ws(tag(","));
    let sep2 = ws(tag(","));
    let (rest, file) = match_file_name(input)?;
    let (rest, extra_args) = opt(preceded(sep, separated_list1(sep2, parse_expr)))(rest)?;
    let file = PathBuf::from(&file.to_string());

    Ok((rest, (file, extra_args)))
}

fn parse_refbin_arg(input: Span) -> IResult<Node> {
    let (rest, (file, extra_args)) = inc_bin_args(input)?;
    let ret =
        Node::new_with_children(IncBinRef(file), extra_args.unwrap_or_default(), span_to_pos( input ));
    Ok((rest, ret))
}

fn parse_write_bin_args(input: Span) -> IResult<Node> {
    use util::ws;
    let (rest, (file_name, _, source_addr, _, size)) = tuple((
        match_file_name,
        ws(tag(",")),
        parse_expr,
        ws(tag(",")),
        parse_expr,
    ))(input)?;

    let file_name = PathBuf::from(file_name.to_string());

    let ret =
        Node::new_with_children(WriteBin(file_name), vec![source_addr, size], span_to_pos(input));

    Ok((rest, ret))
}

fn parse_incbin_arg(input: Span) -> IResult<Node> {
    let (rest, (file, extra_args)) = inc_bin_args(input)?;

    let ret =
        Node::new_with_children(IncBin(file), extra_args.unwrap_or_default(), span_to_pos(input));

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
fn parse_scope_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = get_just_label(input)?;
    let ret = Node::from_item_span(Scope(matched.to_string()), input);
    Ok((rest, ret))
}

fn mk_fill(input: Span, cv: (Node, Node)) -> Node {
    Node::new_with_children(Fill, vec![cv.0, cv.1], span_to_pos( input ))
}

fn parse_bsz_arg(input: Span) -> IResult<Node> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let two_args = separated_pair(parse_expr, sep, parse_expr);
    let one_arg = map(parse_expr, |x: Node| (x, Node::from_number(0, crate::item::ParsedFrom::FromExpr , input)));
    let (rest, matched) = alt((two_args, one_arg))(input)?;
    let ret = mk_fill(input, matched);
    Ok((rest, ret))
}

lazy_static! {
    static ref PARSE_ARG: HashMap<&'static str, CommandParseFn> = {
        let v: Vec<(_, CommandParseFn)> = vec![
            ("scope", parse_scope_arg),
            ("grabmem", parse_grab_mem),
            ("put", parse_put_arg),
            ("writebin", parse_write_bin_args),
            ("incbin", parse_incbin_arg),
            ("incbinref", parse_refbin_arg),
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
            ("exec_addr", parse_exec_arg),
            ("require", parse_require_arg),
        ];
        v.into_iter().collect()
   };
}

fn command_token_function(input: Span) -> IResult<(Span, CommandParseFn)> {
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
