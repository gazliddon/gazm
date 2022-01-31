use crate::expr::parse_expr;
use crate::util::{self, get_block, sep_list1, wrapped_chars, ws};
use crate::labels::{get_just_label, parse_just_label};
use crate::locate::{ Span, matched_span };

use nom::{
    branch::alt,
    bytes::complete::{is_not, take_until},
    character::complete::{line_ending, multispace0, multispace1},
    combinator::{all_consuming, eof, not, opt, recognize},
    multi::{many0, many1},
    sequence::{pair, preceded, separated_pair, terminated},
    AsBytes,
};

use crate::error::{IResult, ParseError, UserError};

use crate::item::{Node,Item};


////////////////////////////////////////////////////////////////////////////////
// Macros
pub fn get_macro_def(input: Span<'_>) -> IResult<(Span, Vec<Node>, Span )> {
    use nom::bytes::complete::tag;
    let rest = input;
    let (rest, (_, name )) = ws(separated_pair(tag("macro"), multispace1, get_just_label))(rest)?;
    let (rest, params) = wrapped_chars('(', sep_list1(parse_just_label), ')')(rest)?;
    let (rest, body) = get_block(rest)?;
    Ok((rest, ( name, params, body )))
}

pub fn parse_macro_definition(input: Span<'_>) -> IResult<Node> {

    let (rest, (name,params, body)) = get_macro_def(input)?;

    let matched_span = matched_span(input, rest);

    let ret = Node::from_item_span(
        Item::MacroDef(name.to_string(), body.to_string()), matched_span
    ).with_children(params);

    Ok((rest, ret))
}
 
pub fn parse_macro_call(input: Span<'_>) -> IResult<Node> {
    let args = wrapped_chars('(', sep_list1(parse_expr), ')');

    let rest = input;
    let (rest, (name, args)) = separated_pair(parse_just_label, multispace0, args) (rest)?;

    println!("Found macro invocation!");

    let matched_span = matched_span(input, rest);
    let ret = Node::from_item_span(
        Item::MacroCall(name.to_string()), matched_span
    ).with_children(args);

    Ok((rest,ret))
}

pub fn expand_macro() {

}
