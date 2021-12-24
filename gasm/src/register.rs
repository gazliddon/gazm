use super::util;
use super::item::{ Item,Node };
use super::ctx::Ctx;

use nom::branch::alt;
use nom::IResult;
use nom::bytes::complete::tag_no_case;
use nom::error::ParseError;
use nom::sequence::{separated_pair, tuple};
use nom::multi::separated_list1;
use nom::character::complete::multispace0;

use nom::bytes::complete::tag;

use std::collections::HashSet;

// Register parsing

pub fn get_reg(input: &str) -> IResult<&str, emu::cpu::RegEnum> {
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

    let matched_lower = String::from(matched).to_ascii_lowercase();

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
fn get_reg_list(input: &str) -> IResult<&str, Vec<emu::cpu::RegEnum>> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, matched) = separated_list1(sep, get_reg)(input)?;
    Ok((rest, matched))
}
fn get_reg_set(input: &str) -> IResult<&str, HashSet<emu::cpu::RegEnum>> {
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

pub fn parse_reg(input: &str) -> IResult<&str, Node> {
    let (rest,matched) = get_reg(input)?;
    Ok((rest, 
        Node::from_item(Item::Register(matched))))
}

pub fn parse_reg_list(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = get_reg_list(input)?;
    Ok((rest, Item::RegisterList(matched)))
}

pub fn parse_reg_list_2_or_more(input: & str) -> IResult<& str, Node> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, (x,xs)) = separated_pair(get_reg, sep, get_reg_list)(input)?;
    let mut xs = xs;
    xs.push(x);
    Ok((rest, 
        Node::from_item(Item::RegisterList(xs))))
}

fn parse_reg_set(_input: &str) -> IResult<&str, Item> {
    todo!()
}

#[allow(unused_imports)]
mod test {

    use pretty_assertions::{assert_eq, assert_ne};
    use super::*;

    #[test]
    fn test_register() {
        let res = parse_reg("A");
        let des = emu::cpu::RegEnum::A;

        let des = Node::from_item(Item::Register(des));
        assert_eq!(res, Ok(("", des)));

        let res = parse_reg("DP");
        let des = emu::cpu::RegEnum::DP;
        let des = Node::from_item(Item::Register(des));
        assert_eq!(res, Ok(("", des)));

    }

    #[test]
    fn test_register_list() {
        use emu::cpu::RegEnum::*;

        let res = parse_reg_list("A,X,Y");
        let des = vec![A,X,Y];
        assert_eq!(res, Ok(("", Item::RegisterList(des))));

        let res = parse_reg_list("");
        assert!(res.is_err());

        let res = parse_reg_list("A");
        let des = vec![A];
        assert_eq!(res, Ok(("", Item::RegisterList(des))));

        let res = parse_reg_list("A, x, y, u, S, DP, cc, D, dp");
        let des = vec![A, X, Y, U, S, DP, CC, D, DP];
        assert_eq!(res, Ok(("", Item::RegisterList(des))));

        let res = parse_reg_list("x,y,u");
        let des = vec![X,Y,U];
        assert_eq!(res, Ok(("", Item::RegisterList(des))));

        let res = parse_reg_list("a,b,d,x,y");
        let des = vec![A,B,D,X,Y];
        assert_eq!(res, Ok(("", Item::RegisterList(des))));


    }

}
