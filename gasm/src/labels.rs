use crate::item::{ Item, Node };
use crate::{ command_token, opcode_token };
use nom::IResult;
use nom::sequence::pair;
use nom::combinator::{ recognize, not};

use nom::character::complete::{
    alpha1, alphanumeric1, 
};

use nom::multi::many0 ;
use nom::bytes::complete::is_a;
use nom::branch::alt;

////////////////////////////////////////////////////////////////////////////////
// Labels
static LOCAL_LABEL_PREFIX: &str = "@!";
static OK_LABEL_CHARS: &str = "_?.";

fn get_just_label(input: &str) -> IResult<&str, &str> {
    // match a label identifier
    let (rest,matched) = recognize(pair(
            alt((alpha1, is_a(OK_LABEL_CHARS))),
            many0(alt((alphanumeric1, is_a(OK_LABEL_CHARS)))),
            ))(input)?;

    // opcodes and commands are reserved
    not( alt((opcode_token, command_token))
       )(matched)?;

    Ok((rest, matched))
}

fn get_local_label(input: &str) -> IResult<&str, &str> {
    let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    let prefix_parse = recognize(pair(loc_tags, get_just_label));

    let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    let postfix_parse = recognize(pair( get_just_label, loc_tags));
    alt((postfix_parse, prefix_parse))(input)
}

fn parse_just_label(input: &str) -> IResult<&str, Node> {
    let (rest, matched) = get_just_label(input)?;
    Ok((rest, 
        Node::from_item(
        Item::Label(matched.to_string()))))
}

fn parse_local_label(input: &str) -> IResult<&str, Node> {
    let (rest, matched) = get_local_label(input)?;
    Ok((rest,
        Node::from_item(
        Item::LocalLabel(matched.to_string()))))
}

pub fn parse_label(input: &str) -> IResult<&str, Node> {
    alt((parse_local_label, parse_just_label))(input)
}

type PResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    msg : String
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        panic!()
    }
    fn description(&self) -> &str { &self.msg }
    fn cause(&self) -> Option<&dyn std::error::Error> { panic!() }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { todo!() }
}

impl From<nom::Err<nom::error::Error<&str>>> for ParseError {
    fn from(e : nom::Err<nom::error::Error<&str>>) -> Self {
        Self {
            msg : e.to_string()
        }
    }
}

#[allow(unused_imports)]
mod test {
    use crate::commands::parse_command;

    use super::Item;

    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_parse_label() {
        let nl = "non_local".to_string();
        let res = parse_label(&nl);
        assert_eq!(res, Ok(("", Item::Label(nl.clone()))));
        let res = parse_label("adc");
        assert_eq!(res, Ok(("", Item::Label("adc".to_string()))));
    }
    fn mk_label(a : &str) -> Item {
        Item::Label(a.to_string())
    }
    fn mk_loc_label(a : &str) -> Item {
        Item::LocalLabel(a.to_string())
    }

    #[test]
    fn test_parse_local_label() {
        let lab_str = "@_local";
        let res = parse_label(lab_str);
        let des = mk_loc_label(lab_str);
        assert_eq!(res, Ok(("", des)));


        let lab_str = "local@";
        let res = parse_label(lab_str);
        let des = mk_loc_label(lab_str);
        assert_eq!(res, Ok(("", des)));

        let lab_str = "local!";
        let res = parse_label(lab_str);
        let des = mk_loc_label(lab_str);
        assert_eq!(res, Ok(("", des)));

        let lab_str = "!local_6502";
        let res = parse_label(lab_str);
        let des = mk_loc_label(lab_str);
        assert_eq!(res, Ok(("", des)));
    }

    #[test]
    fn test_label_no_opcodes() {
        let res = parse_label("NEG");
        assert_ne!(res, Ok(("",  Item::Label("NEG".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("neg");
        assert_ne!(res, Ok(("",  Item::Label("neg".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("negative");
        assert_eq!(res, Ok(("",  Item::Label("negative".to_string()) )) );
    }

    #[test]
    fn test_label_no_commands() {
        let res = parse_label("fdb");
        assert_ne!(res, Ok(("",  Item::Label("fdb".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("org");
        assert_ne!(res, Ok(("",  Item::Label("org".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("!org");
        assert_ne!(res, Ok(("",  Item::LocalLabel("org".to_string()) )) );
        assert!(res.is_err());

        let res = parse_label("equation");
        assert_eq!(res, Ok(("",  Item::Label("equation".to_string()) )) );
    }
}
