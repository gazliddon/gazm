use nom::{
    branch::alt,
    bytes::complete::{is_not, take_until},
    character::complete::{line_ending, multispace0, multispace1},
    combinator::{all_consuming, eof, not, opt, recognize},
    multi::{many0, many1},
    sequence::{pair, preceded, separated_pair, terminated},
    AsBytes,
};

use crate::{
    cli, commands, comments,
    expr::{self, parse_expr},
    item,
    labels::{get_just_label, parse_label},
    macros::{parse_macro_call, parse_macro_definition},
    messages, opcodes,
    util::{self, sep_list1, wrapped_chars, ws},
};

use crate::error::{IResult, ParseError, UserError};
use crate::item::{Item, Node};
use crate::locate::Span;
pub fn parse_block(input: Span<'_>) -> IResult<Span> {
    ws(wrapped_chars('{', is_not("}"), '}'))(input)
}

pub fn get_struct(input: Span<'_>) -> IResult<Span> {
    // label
    // load of stuff between braces
    use nom::bytes::complete::tag;

    let rest = input;

    let (rest, (_, _name)) = ws(separated_pair(tag("struct"), multispace1, get_just_label))(rest)?;

    let (rest, body) = parse_block(rest)?;

    Ok((rest, body))
}

