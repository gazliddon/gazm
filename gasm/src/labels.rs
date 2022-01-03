use crate::item::{ Item, Node };
use crate::opcodes::opcode_just_token;
use crate::{ commands::command_token, opcodes::opcode_token };
use nom::sequence::pair;
use nom::combinator::{ recognize, not};

use nom::character::complete::{
    alpha1, alphanumeric1, 
};

use nom::multi::many0 ;
use nom::bytes::complete::is_a;
use nom::branch::alt;

use crate::error::IResult;
use crate::locate::{ Span,mk_span };

use nom_locate::position;

////////////////////////////////////////////////////////////////////////////////
// Labels
static LOCAL_LABEL_PREFIX: &str = "@!";
static OK_LABEL_CHARS: &str = "_?.";

fn get_just_label(input: Span) -> IResult<Span> {
    // match a label identifier
    let (rest,matched) = recognize(pair(
            alt((alpha1, is_a(OK_LABEL_CHARS))),
            many0(alt((alphanumeric1, is_a(OK_LABEL_CHARS)))),
            ))(input)?;


    // opcodes and commands are reserved
    not( alt((opcode_just_token, command_token))
       )(matched)?;

    Ok((rest, matched))
}

fn get_local_label(input: Span) -> IResult<Span> {
    let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    let prefix_parse = recognize(pair(loc_tags, get_just_label));

    let loc_tags = is_a(LOCAL_LABEL_PREFIX);
    let postfix_parse = recognize(pair( get_just_label, loc_tags));

    alt((postfix_parse, prefix_parse))(input)
}

fn parse_just_label(input: Span) -> IResult<Node> {
    let (rest, matched) = get_just_label(input)?;
    let ret = Node::from_item(Item::Label(matched.to_string()), matched);
    Ok((rest,ret))
}

fn parse_local_label(input: Span) -> IResult< Node> {
    let (rest, matched) = get_local_label(input)?;
    let ret = Node::from_item(Item::LocalLabel(matched.to_string()), matched);
    Ok((rest,ret))
}

pub fn parse_label(input: Span) -> IResult<Node> {
    let (rest, matched) = alt((parse_local_label, parse_just_label))(input)?;
    Ok((rest,matched))
}

#[allow(unused_imports)]
mod test {
    use crate::locate::{mk_span, Position};
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    #[test]
    fn test_parse_label() {
        let input = Span::new("hello ;;");
        let (rest,matched) = parse_label(input).unwrap();
        assert_eq!(" ;;", *rest);
        assert_eq!("hello",&matched.to_string());
    }

    // fn prep_label<'a>(nl : &'a str) -> IResult<(Node, Node)> {
    //     let des_pos = Position::new(0,nl.len());
    //     let nl = mk_span("test", nl);
    //     let (rest, matched) = parse_label(nl)?;
    //     let des = Node::to_label(&nl.to_string()).with_ctx(des_pos);
    //     Ok((rest, (matched, des)))
    // }

    // fn prep_loc_label<'a>(nl : &'a str) -> IResult<(Node, Node)> {
    //     let des_pos = Position::new(0,nl.len());
    //     let nl = mk_span("test", nl);
    //     let (rest, matched) = parse_label(nl)?;
    //     let des = Node::to_local_lable(&nl.to_string()).with_ctx(des_pos);
    //     Ok((rest, (matched, des)))
    // }

    // #[test]
    // fn test_parse_label() {
    //     let nl = "non_local";
    //     let (_,( res,des ))= prep_label(nl).unwrap();
    //     assert_eq!(res, des);

    //     let nl = "abc";
    //     let (_,( res,des ))= prep_label(nl.into()).unwrap();
    //     assert_eq!(res, des);
    // }

    // #[test]
    // fn test_parse_local_label() {

    //     let nl = "@_local";
    //     let res = prep_loc_label(nl);
    //     assert!(res.is_ok());
    //     let (_,(res,des)) = res.unwrap();
    //     assert_eq!(res,des);

    //     let nl = "local@";
    //     let res = prep_loc_label(nl);
    //     assert!(res.is_ok());
    //     let (_,(res,des)) = res.unwrap();
    //     assert_eq!(res,des);

    //     let nl = "!_local";
    //     let res = prep_loc_label(nl);
    //     assert!(res.is_ok());
    //     let (_,(res,des)) = res.unwrap();
    //     assert_eq!(res,des);

    //     let nl = "local!";
    //     let res = prep_loc_label(nl);
    //     assert!(res.is_ok());
    //     let (_,(res,des)) = res.unwrap();
    //     assert_eq!(res,des);

    //     let nl = "!local_6502";
    //     let res = prep_loc_label(nl);
    //     assert!(res.is_ok());
    //     let (_,(res,des)) = res.unwrap();
    //     assert_eq!(res,des);
    // }



    // #[test]
    // fn test_label_no_opcodes() {
    //     let nl  = "lda";
    //     let res = prep_label(nl);
    //     assert!(res.is_err());

    //     let nl  = "LDA";
    //     let res = prep_label(nl);
    //     assert!(res.is_err());

    //     let nl  = "STA";
    //     let res = prep_label(nl);
    //     assert!(res.is_err());

    //     let nl  = "STAmina";
    //     let (_, (res,des)) = prep_label(nl).unwrap();
    //     assert_eq!(res,des)
    // }

    // #[test]
    // fn test_label_no_commands() {
    //     let nl  = "FDB";
    //     let res = prep_label(nl);
    //     assert!(res.is_err());

    //     let nl  = "fdb";

    //     let res = prep_label(nl);
    //     assert!(res.is_err());

    //     let nl  = "org";
    //     let res = prep_label(nl);
    //     assert!(res.is_err());

    //     let nl  = "organize";
    //     let (_, (res,des)) = prep_label(nl).unwrap();
    //     assert_eq!(res,des)
    // }
}
