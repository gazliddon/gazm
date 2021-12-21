use crate::Item;
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

fn get_label_identifier(input: &str) -> IResult<&str, &str> {
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

fn get_label(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = get_label_identifier(input)?;
    Ok((rest, Item::Label(matched.to_string())))
}

fn get_local_label(input: &str) -> IResult<&str, Item> {
    let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    let prefix_parse = recognize(pair(loc_tags, get_label_identifier));

    let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    let postfix_parse = recognize(pair( get_label_identifier, loc_tags));

    let (rest, matched) = alt((postfix_parse, prefix_parse))(input)?;
    Ok((rest, Item::LocalLabel(matched.to_string())))
}

pub fn parse_label(input: &str) -> IResult<&str, Item> {
    alt((get_local_label, get_label))(input)
}

#[allow(unused_imports)]
mod test {
    use crate::commands::parse_command;

    use super::*;
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
