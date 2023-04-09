use crate::commands::command_token;
use crate::item::{Item, Node};
use crate::opcodes::opcode_just_token;
use nom::combinator::{all_consuming, recognize};
use nom::sequence::pair;

use nom::character::complete::{alpha1, alphanumeric1};

use nom::branch::alt;
use nom::bytes::complete::is_a;
use nom::multi::many0;

use crate::error::IResult;
use crate::locate::Span;
            use crate::item::LabelDefinition;

////////////////////////////////////////////////////////////////////////////////
// Labels
static LOCAL_LABEL_PREFIX: &str = "@!";
static OK_LABEL_CHARS: &str = "_?.";

pub fn get_just_label(input: Span) -> IResult<Span> {
    use crate::error::parse_error;
    // match a label identifier
    let (rest, matched) = recognize(pair(
        alt((alpha1, is_a(OK_LABEL_CHARS))),
        many0(alt((alphanumeric1, is_a(OK_LABEL_CHARS)))),
    ))(input)?;

    // make sure it's not a command or opcode
    if all_consuming(alt((command_token, opcode_just_token)))(matched).is_ok() {
        // let msg = format!(
        //     "{} is a reserved keyword and cannot be used as a label",
        //     matched
        // );
        let msg = "Keyword";
        Err(parse_error(msg, matched))
    } else {
        Ok((rest, matched))
    }
}

fn get_local_label(input: Span) -> IResult<Span> {
    let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    let mut prefix_parse = recognize(pair(loc_tags, get_just_label));

    // let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    // let postfix_parse = recognize(pair(get_just_label, loc_tags));

    // alt((postfix_parse, prefix_parse))(input)
    prefix_parse(input)
}

pub fn parse_just_label(input: Span) -> IResult<Node> {
    let (rest, matched) = get_just_label(input)?;
    let ret = Node::from_item_span(Item::Label(
            LabelDefinition::Text(matched.to_string())
            ), matched);
    Ok((rest, ret))
}

fn parse_local_label(input: Span) -> IResult<Node> {
    let (rest, matched) = get_local_label(input)?;
    let ret = Node::from_item_span(Item::LocalLabel(
            LabelDefinition::Text(matched.to_string()))
            , matched);
    Ok((rest, ret))
}

pub fn parse_label(input: Span) -> IResult<Node> {
    let (rest, matched) = alt((parse_local_label, parse_just_label))(input)?;
    Ok((rest, matched))
}

#[allow(unused_imports)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};
    use emu::utils::sources::{AsmSource, Position};

    use super::*;

    #[test]
    fn test_parse_label() {
        let input = Span::new_extra("hello ;;", AsmSource::FromStr);
        let (rest, matched) = parse_label(input).unwrap();
        assert_eq!(" ;;", *rest);
        assert_eq!("hello", &matched.to_string());
    }
    #[test]
    fn test_parse_opcode_like_lable() {
        let input = Span::new_extra("swi3_vec ;;", AsmSource::FromStr);
        let (rest, matched) = parse_label(input).unwrap();
        assert_eq!(" ;;", *rest);
        assert_eq!("swi3_vec", &matched.to_string());
    }

    #[test]
    fn test_parse_local_abel() {
        let input = Span::new_extra("@hello\n", AsmSource::FromStr);
        let (rest, matched) = parse_label(input).unwrap();
        assert_eq!("\n", *rest);
        assert_eq!("@hello", &matched.to_string());
    }
}
