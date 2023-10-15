#![deny(unused_imports)]

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case},
    character::complete::multispace1,
    combinator::{map, opt},
    multi::separated_list0,
    sequence::separated_pair,
};

use crate::{
    error::IResult,
    item::{Item, Node, StructMemberType},
};

use super:: {
    comments::strip_comments,
    util::{wrapped_chars, ws},
    labels::get_just_label,
    locate::{matched_span, span_to_pos, Span},
};


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
    let (rest, item_type) = alt((
        map(tag_no_case("byte"), |_| StructMemberType::Byte),
        map(tag_no_case("word"), |_| StructMemberType::Word),
        map(tag_no_case("dword"), |_| StructMemberType::DWord),
        map(tag_no_case("qword"), |_| StructMemberType::QWord),
        map(get_just_label, |utype| {
            StructMemberType::UserType(utype.to_string())
        }),
    ))(input)?;

    Ok((rest, item_type))
}

fn get_struct_entry(
    input: Span<'_>,
) -> IResult<(Span, StructMemberType, Option<Span>, Option<Node>)> {
    let (input, comment) = strip_comments(input)?;

    let sep = ws(tag(":"));
    let (rest, (name, item_type)) =
        separated_pair(get_just_label, sep, get_struct_arg_type)(input)?;
    let mut array = opt(ws(wrapped_chars('[', ws(is_not("]")), ']')));
    let (rest, an_array) = array(rest)?;
    Ok((rest, (name, item_type, an_array, comment)))
}

fn parse_struct_entry(input: Span<'_>) -> IResult<Node> {
    use crate::parse::expr::parse_expr;

    let (rest, (name, entry_type, array_exp, _)) = get_struct_entry(input)?;
    let size = entry_type.to_size_item();

    let multiplier = if let Some(expr) = array_exp {
        let (_, matched) = parse_expr(expr)?;
        matched
    } else {
        Node::from_item_span(Item::Num(1, crate::item::ParsedFrom::FromExpr), input)
    };

    let children = vec![
        multiplier,
        Node::from_item_span(Item::Mul, input),
        Node::from_item_span(size, input),
    ];

    let expr = Node::new_with_children(Item::Expr, &children, span_to_pos(input));
    let node = Node::from_item_span(Item::StructEntry(name.to_string()), input).with_child(expr);

    Ok((rest, node))
}

pub fn parse_struct_definition(input: Span<'_>) -> IResult<Node> {
    let (rest, (name, body)) = get_struct(input)?;

    let sep = ws(tag(","));
    let (spare, entries) = ws(separated_list0(sep, parse_struct_entry))(body)?;
    let sep = ws(tag(","));
    let (spare, _) = opt(sep)(spare)?;

    if spare.is_empty() {
        let matched_span = matched_span(input, rest);
        let res = Node::new_with_children(
            Item::StructDef(name.to_string()),
            &entries,
            span_to_pos(matched_span),
        );
        Ok((rest, res))
    } else {
        let m = "Unexpected text in struct definition, missing comma on previous line?";
        Err(crate::error::parse_failure(m, spare))
    }
}
