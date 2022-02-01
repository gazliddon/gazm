use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until, tag_no_case},
    character::complete::{line_ending, multispace0, multispace1},
    combinator::{all_consuming, eof, not, opt, recognize, map},
    multi::{many0, many1, separated_list0},
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    AsBytes,
};

use crate::{
    labels::{get_just_label, parse_label},
    locate::matched_span,
    macros::{parse_macro_call, parse_macro_definition},
    util::{self, sep_list1, wrapped_chars, ws},
};

use crate::error::{IResult, ParseError, UserError};
use crate::item::{Item, Node, StructEntry, StructMemberType};
use crate::locate::Span;

fn parse_block(input: Span<'_>) -> IResult<Span> {
    ws(wrapped_chars('{', is_not("}"), '}'))(input)
}

pub fn get_struct(input: Span<'_>) -> IResult<(Span, Span)> {
    let rest = input;
    let (rest, (_, name)) = ws(separated_pair(tag("struct"), multispace1, get_just_label))(rest)?;
    let (rest, body) = parse_block(rest)?;
    Ok((rest, (name, body)))
}

fn get_struct_arg_type(input: Span<'_>) -> IResult<StructMemberType> {
     alt((
            map(tag_no_case("byte"), |_| StructMemberType::Byte),
            map(tag_no_case("word"), |_| StructMemberType::Word),
            map(tag_no_case("dword"), |_| StructMemberType::DWord),
            map(tag_no_case("qword"), |_| StructMemberType::QWord),
            map(get_just_label, |utype| StructMemberType::UserType(utype.to_string())),
    ))(input)
}

fn get_struct_entry(input: Span<'_>) -> IResult<StructEntry> {
    let sep = ws(tag(":"));
    let (rest, (name, item_type)) =
        separated_pair(get_just_label, sep, get_struct_arg_type)(input)?;
    let ret = StructEntry {
        name: name.to_string(),
        item_type,
    };
    Ok((rest, ret))
}

pub fn parse_struct_definition(input: Span<'_>) -> IResult<Node> {
    let (rest, (name, body)) = get_struct(input)?;

    let sep = ws(tag(","));

    let (_, defs) = separated_list0(sep, get_struct_entry)(body)?;
    let matched_span = matched_span(input, rest);
    let res = Node::from_item_span(Item::StructDef(name.to_string(), defs), matched_span);
    Ok((rest, res))
}
