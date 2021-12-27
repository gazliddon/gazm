use super::util;
use super::item::{ Item,Node };
use super::ctx::Ctx;

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::sequence::{separated_pair, tuple};
use nom::multi::separated_list1;
use nom::character::complete::multispace0;

use nom::bytes::complete::tag;

use std::collections::HashSet;

use crate::error::{IResult, ParseError};
use crate::locate::Span;

// Register parsing

pub fn get_reg(input: Span) -> IResult<emu::cpu::RegEnum> {
    let (rest, matched) = alt((
            tag_no_case("pcr"),
            tag_no_case("dp"),
            tag_no_case("cc"),
            tag_no_case("pc"),
            tag_no_case("a"),
            tag_no_case("b"),
            tag_no_case("x"),
            tag_no_case("y"),
            tag_no_case("u"),
            tag_no_case("s"),
            tag_no_case("d")))(input)?;

    use emu::cpu::RegEnum::*;

    let matched_lower = matched.to_string().to_ascii_lowercase();

    let reg = match matched_lower.as_str() {
            "pcr" => PC,
            "dp" => DP,
            "cc" => CC,
            "pc" => PC,
            "a" => A,
            "b" => B,
            "x" => X,
            "y" => Y,
            "u" => U,
            "s" => S,
            "d" => D,
        _ => panic!("Should not happen"),
    };

    Ok((rest, reg))
}
fn get_reg_list(input: Span) -> IResult< Vec<emu::cpu::RegEnum>> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, matched) = separated_list1(sep, get_reg)(input)?;
    Ok((rest, matched))
}

fn get_reg_set(input: Span) -> IResult< HashSet<emu::cpu::RegEnum>> {
    let mut ret = HashSet::new();

    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, matched) = separated_list1(sep, get_reg)(input)?;

    for r in matched {
        if ret.contains(&r) {
            let err = nom::error::make_error(input, nom::error::ErrorKind::Fail);
            return Err( nom::Err::Failure(err))
        } else {
            ret.insert(r);
        }
    }

    Ok((rest, ret))
}

pub fn parse_reg(input: Span) -> IResult< Node> {
    let (rest,matched) = get_reg(input)?;
    Ok((rest, 
        Node::from_item(Item::Register(matched))))
}

pub fn parse_reg_list(input: Span) -> IResult< Item> {
    let (rest, matched) = get_reg_list(input)?;
    Ok((rest, Item::RegisterList(matched)))
}

pub fn parse_reg_set(input: Span) -> IResult< Node> {
    let (rest, matched) = get_reg_set(input)?;
    Ok((rest, Node::from_item(Item::RegisterSet(matched))))
}

pub fn parse_reg_set_2(input: Span) -> IResult< Node> {

    use nom::error::ErrorKind::NoneOf;
    use nom::error::Error;

    let (rest,matched) = parse_reg_set(input)?;

    if let Item::RegisterSet(regs) = matched.item() {
        if regs.len() < 2 {
            return 

        Err(nom::Err::Error(ParseError::new(
            "Need at least 2 registers in list".to_owned(),
            input,
        )));


        }
    } 

    Ok((rest,matched))
}

#[allow(unused_imports)]
mod test {

    use pretty_assertions::{assert_eq, assert_ne};
    use super::*;

    #[test]
    fn test_register() {
        let res = parse_reg("A".into());
        let des = emu::cpu::RegEnum::A;

        let des = Node::from_item(Item::Register(des));
        assert_eq!(res, Ok(("".into(), des)));

        let res = parse_reg("DP".into());
        let des = emu::cpu::RegEnum::DP;
        let des = Node::from_item(Item::Register(des));
        assert_eq!(res, Ok(("".into(), des)));

    }

    #[test]
    fn test_register_list() {
        use emu::cpu::RegEnum::*;

        let res = parse_reg_list("A,X,Y".into());
        let des = vec![A,X,Y];
        assert_eq!(res, Ok(("".into(), Item::RegisterList(des))));

        let res = parse_reg_list("".into());
        assert!(res.is_err());

        let res = parse_reg_list("A".into());
        let des = vec![A];
        assert_eq!(res, Ok(("".into(), Item::RegisterList(des))));

        let res = parse_reg_list("A, x, y, u, S, DP, cc, D, dp".into());
        let des = vec![A, X, Y, U, S, DP, CC, D, DP];
        assert_eq!(res, Ok(("".into(), Item::RegisterList(des))));

        let res = parse_reg_list("x,y,u".into());
        let des = vec![X,Y,U];
        assert_eq!(res, Ok(("".into(), Item::RegisterList(des))));

        let res = parse_reg_list("a,b,d,x,y".into());
        let des = vec![A,B,D,X,Y];
        assert_eq!(res, Ok(("".into(), Item::RegisterList(des))));
    }

}
