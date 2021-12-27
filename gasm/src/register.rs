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

pub fn parse_reg_set_2(input: Span) -> IResult<Node> {

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

    use lazy_static::__Deref;
    use pretty_assertions::{assert_eq, assert_ne};
    use super::*;

    #[test]
    fn test_register() {
        use emu::cpu::RegEnum::*;

        let reg = "a";
        let des = A;

        let res = parse_reg(reg.into());
        assert!(res.is_ok());
        let (_,res) = res.unwrap();
        assert_eq!(*res.item(),Item::Register(des));

        let reg = "dp";
        let des = DP;

        let res = parse_reg(reg.into());
        assert!(res.is_ok());
        let (_,res) = res.unwrap();
        assert_eq!(*res.item(),Item::Register(des));
    }

    fn test_reg_list(input : &str, des : Vec<emu::cpu::RegEnum>)  {
        println!("Input: {}", input);
        println!("Desired: {:?}", des);
        let res = parse_reg_list(input.into());
        assert!(res.is_ok());
        let (rest, matched) = res.unwrap();
        let rest : &str = rest.deref();
        assert_eq!(matched, Item::RegisterList(des));
        assert_eq!("", rest);
    }

    fn test_reg_set(input : &str, des : HashSet<emu::cpu::RegEnum>)  {
        let des = Node::from_item(Item::RegisterSet(des));
        println!("Input: {}", input);
        println!("Desired: {:?}", des);
        let res = parse_reg_set_2(input.into());
        assert!(res.is_ok());
        let (rest, matched) = res.unwrap();
        let rest : &str = rest.deref();
        assert_eq!(matched.item(), des.item());
        assert_eq!("", rest);
    }
    #[test]
    fn test_register_set() {
        use emu::cpu::RegEnum::*;

        let input = "A,X,Y";
        let des = HashSet::from([A,X,Y]);
        test_reg_set(input, des);

        let input = "A,B";
        let des = HashSet::from([B,A]);
        test_reg_set(input, des);

        let input = "A, x, y, u, S, DP, cc, D";
        let des = HashSet::from([A, X, Y, U, S, DP, CC, D]);
        test_reg_set(input, des);

        let input = "x,y,u";
        let des = HashSet::from([X,Y,U]);
        test_reg_set(input, des);

        let input = "a,b,d,x,y";
        let des = HashSet::from([A,B,D,X,Y]);
        test_reg_set(input, des);
    }

    #[test]
    fn test_register_list() {
        use emu::cpu::RegEnum::*;

        let input = "A,X,Y";
        let des = vec![A,X,Y];
        test_reg_list(input, des);

        let input = "A";
        let des = vec![A];
        test_reg_list(input, des);

        let input = "A, x, y, u, S, DP, cc, D, dp";
        let des = vec![A, X, Y, U, S, DP, CC, D, DP];
        test_reg_list(input, des);

        let input = "x,y,u";
        let des = vec![X,Y,U];
        test_reg_list(input, des);

        let input = "a,b,d,x,y";
        let des = vec![A,B,D,X,Y];
        test_reg_list(input, des);
    }

}
