use crate::{
    error::parse_error,
    error::IResult,
    item::{Item, LabelDefinition::Text, LabelDefinition::TextScoped, Node},
    parse6809::opcodes::opcode_just_token,
};

use super::{commands::command_token, locate::Span};

use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    character::complete::{alpha1, alphanumeric1},
    combinator::{all_consuming, opt, recognize},
    multi::{many0, many1},
    sequence::{pair, preceded},
};

////////////////////////////////////////////////////////////////////////////////
// Labels
static LOCAL_LABEL_PREFIX: &str = "@!";
static OK_LABEL_CHARS: &str = "_?.";

// scoped symbol is just a symbol or!
// opt(symbol)sep(symbol+)
#[allow(dead_code)]
pub fn get_scoped_label(input: Span) -> IResult<Span> {
    let (rest, matched) = recognize(pair(
        opt(get_just_label),
        many1(preceded(tag("::"), get_just_label)),
    ))(input)?;
    Ok((rest, matched))
}

pub fn get_just_label(input: Span) -> IResult<Span> {
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
    let node = Node::from_item_span(Item::Label(Text(matched.to_string())), matched);
    Ok((rest, node))
}

fn parse_local_label(input: Span) -> IResult<Node> {
    let (rest, matched) = get_local_label(input)?;
    let node = Node::from_item_span(Item::LocalLabel(Text(matched.to_string())), matched);
    Ok((rest, node))
}

fn parse_scoped_label(input: Span) -> IResult<Node> {
    let (rest, matched) = get_scoped_label(input)?;
    let node = Node::from_item_span(Item::Label(TextScoped(matched.to_string())), matched);
    Ok((rest, node))
}

pub fn parse_label(input: Span) -> IResult<Node> {
    let (rest, matched) = alt((parse_scoped_label, parse_local_label, parse_just_label))(input)?;
    Ok((rest, matched))
}

#[allow(unused_imports)]
mod test {
    use sources::{AsmSource, Position};
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;
    #[test]
    fn scope_label() {
        let to_test = vec!["hello::campers::chums", "::test"];

        for lab in to_test {
            let input = Span::new_extra(lab, AsmSource::FromStr);
            let (rest, matched) = get_scoped_label(input).expect("A scoped label");
            let matched = matched.to_string();
            assert_eq!(matched, lab);
            assert!(rest.is_empty());
        }
    }

    #[test]
    fn test_parse_label() {
        let test_data = vec!["@pooo", "hello::campers::chums", "test", "!spanner"];

        for label_text in test_data {
            let extra_text = "  ;;";
            let lab = format!("{label_text}{extra_text}");
            let input = Span::new_extra(&lab, AsmSource::FromStr);
            let (rest, matched) = parse_label(input).unwrap();
            assert_eq!(extra_text, *rest);
            assert_eq!(label_text, &matched.to_string());
        }
    }
    #[test]
    fn test_parse_opcode_like_label() {
        let input = Span::new_extra("swi3_vec ;;", AsmSource::FromStr);
        let (rest, matched) = parse_label(input).unwrap();
        assert_eq!(" ;;", *rest);
        assert_eq!("swi3_vec", &matched.to_string());
    }

    #[test]
    fn test_parse_local_label() {
        let input = Span::new_extra("@hello\n", AsmSource::FromStr);
        let (rest, matched) = parse_label(input).unwrap();
        assert_eq!("\n", *rest);
        assert_eq!("@hello", &matched.to_string());
    }
}
