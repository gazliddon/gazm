#![deny(unused_imports)]
use crate::{ item::{Item, Node},error::IResult };

use nom::{
    branch::alt,
    bytes::complete::tag,
    bytes::complete::take_until,
    character::complete::not_line_ending,
    combinator::recognize,
    sequence::{preceded, tuple},
};

use super::locate::Span;

static COMMENT: &str = ";";

fn get_comment(input: Span) -> IResult<Span> {
    let comments = alt((tag(";"), tag("//")));
    preceded(comments, recognize(not_line_ending))(input)
}

fn get_star_comment(input: Span) -> IResult<Span> {
    let comments = tag("*");
    preceded(comments, recognize(not_line_ending))(input)
}

pub fn parse_comment(input: Span) -> IResult<Node> {
    use Item::*;

    let (rest, matched) = get_comment(input)?;
    let node = Node::from_item_span(Comment(matched.to_string()), input);
    Ok((rest, node))
}
pub fn parse_star_comment(input: Span) -> IResult<Node> {
    use Item::*;
    let (rest, matched) = get_star_comment(input)?;
    let node = Node::from_item_span(Comment(matched.to_string()), input);
    Ok((rest, node))
}

// Strips comment if there
pub fn strip_comments(input: Span) -> IResult<Option<Node>> {
    let not_comment_1 = take_until(COMMENT);
    let not_comment_2 = take_until("//");
    let not_comment = alt((not_comment_2, not_comment_1));

    let res = tuple((not_comment, parse_comment))(input);

    if let Ok((_rest, (pre_comment, node))) = res {
        Ok((pre_comment, Some(node)))
    } else {
        Ok((input, None))
    }
}

////////////////////////////////////////////////////////////////////////////////
// tests
#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use grl_sources::Position;

    use super::*;
    use pretty_assertions::assert_eq;

}
