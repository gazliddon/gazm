use crate::{
    error::IResult,
    item::{Item, Node},
    parse::locate::Span,
    parse::util,
    item6809::MC6809::{self,RegisterSet},
};

use emu6809::cpu::RegEnum;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, multispace0},
    multi::separated_list1,
    sequence::{separated_pair, tuple},
};

use std::collections::HashSet;

// Register parsing

pub fn get_reg(input: Span) -> IResult<RegEnum> {
    use RegEnum::*;

    let (rest, input) = alpha1(input)?;
    let cmp = input.to_lowercase();

    let reg = match cmp.as_str() {
        "pcr" | "pc" => PC,
        "dp" => DP,
        "cc" => CC,
        "a" => A,
        "b" => B,
        "x" => X,
        "y" => Y,
        "u" => U,
        "s" => S,
        "d" => D,
        _ => {
            let msg = format!("Expecting a register: {cmp}");
            return Err(crate::error::parse_error(&msg, input));
        }
    };

    Ok((rest, reg))
}

pub fn get_index_reg(input: Span) -> IResult<RegEnum> {
    let (rest, reg) = get_reg(input)?;

    if reg.is_valid_for_index() {
        Ok((rest, reg))
    } else {
        let msg = format!("Illegal index register {reg:?}, must be either: X, Y, S, U",);
        Err(crate::error::parse_failure(&msg, input))
    }
}

pub fn get_pc_reg(input: Span) -> IResult<RegEnum> {
    let (rest, reg) = get_reg(input)?;
    if reg == RegEnum::PC {
        Ok((rest, reg))
    } else {
        Err(crate::error::parse_error("expected PC", input))
    }
}

pub fn get_reg_pair(input: Span) -> IResult<(RegEnum, RegEnum)> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, matched) = separated_pair(get_reg, sep, get_reg)(input)?;
    Ok((rest, matched))
}

fn get_reg_set(input: Span) -> IResult<HashSet<RegEnum>> {
    let mut hash_ret = HashSet::new();

    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, matched) = separated_list1(sep, get_reg)(input)?;

    for r in matched {
        if hash_ret.contains(&r) {
            let err = nom::error::make_error(input, nom::error::ErrorKind::Fail);
            return Err(nom::Err::Error(err));
        }
        hash_ret.insert(r);
    }

    Ok((rest, hash_ret))
}

pub fn parse_reg_set_n(input: Span, n: usize) -> IResult<Node> {
    let (rest, matched) = parse_reg_set(input)?;

    if let Item::Cpu(RegisterSet(regs)  )= &matched.item {
        if regs.len() < n {
            return Err(crate::error::parse_error(
                "Need at least 2 registers in list",
                input,
            ));
        }
    }

    Ok((rest, matched))
}

fn parse_reg_set(input: Span) -> IResult<Node> {
    let (rest, matched) = get_reg_set(input)?;

    let node = Node::from_item_span(RegisterSet(matched), input);
    Ok((rest, node))
}

pub fn parse_reg_set_1(input: Span) -> IResult<Node> {
    parse_reg_set_n(input, 1)
}

#[allow(unused_imports)]
mod test {

    use super::*;
    use lazy_static::__Deref;
    use pretty_assertions::{assert_eq, assert_ne};

    // #[test]
    // fn test_register() {
    //     use emu::cpu::RegEnum::*;

    //     let reg = "a";
    //     let des = A;

    //     let res = parse_reg(reg.into());
    //     assert!(res.is_ok());
    //     let (_,res) = res.unwrap();
    //     assert_eq!(*res.item(),Item::Register(des));

    //     let reg = "dp";
    //     let des = DP;

    //     let res = parse_reg(reg.into());
    //     assert!(res.is_ok());
    //     let (_,res) = res.unwrap();
    //     assert_eq!(*res.item(),Item::Register(des));
    // }

    // fn test_reg_list(input : &str, des : Vec<emu::cpu::RegEnum>)  {
    //     println!("Input: {}", input);
    //     println!("Desired: {:?}", des);
    //     let input = mk_span("test", input);
    //     let res = parse_reg_list(input);
    //     assert!(res.is_ok());
    //     let (rest, matched) = res.unwrap();
    //     let rest : &str = rest.deref();
    //     assert_eq!(matched, Item::RegisterList(des));
    //     assert_eq!("", rest);
    // }

    // fn test_reg_set(input : &str, des : HashSet<emu::cpu::RegEnum>)  {
    //     let des = Node::from_item(Item::RegisterSet(des));
    //     println!("Input: {}", input);
    //     println!("Desired: {:?}", des);
    //     let input = mk_span("test", input);
    //     let res = parse_reg_set_2(input);
    //     assert!(res.is_ok());
    //     let (rest, matched) = res.unwrap();
    //     let rest : &str = rest.deref();
    //     assert_eq!(matched.item(), des.item());
    //     assert_eq!("", rest);
    // }
    // #[test]
    // fn test_register_set() {
    //     use emu::cpu::RegEnum::*;

    //     let input = "A,X,Y";
    //     let des = HashSet::from([A,X,Y]);
    //     test_reg_set(input, des);

    //     let input = "A,B";
    //     let des = HashSet::from([B,A]);
    //     test_reg_set(input, des);

    //     let input = "A, x, y, u, S, DP, cc, D";
    //     let des = HashSet::from([A, X, Y, U, S, DP, CC, D]);
    //     test_reg_set(input, des);

    //     let input = "x,y,u";
    //     let des = HashSet::from([X,Y,U]);
    //     test_reg_set(input, des);

    //     let input = "a,b,d,x,y";
    //     let des = HashSet::from([A,B,D,X,Y]);
    //     test_reg_set(input, des);
    // }

    // #[test]
    // fn test_register_list() {
    //     use emu::cpu::RegEnum::*;

    //     let input = "A,X,Y";
    //     let des = vec![A,X,Y];
    //     test_reg_list(input, des);

    //     let input = "A";
    //     let des = vec![A];
    //     test_reg_list(input, des);

    //     let input = "A, x, y, u, S, DP, cc, D, dp";
    //     let des = vec![A, X, Y, U, S, DP, CC, D, DP];
    //     test_reg_list(input, des);

    //     let input = "x,y,u";
    //     let des = vec![X,Y,U];
    //     test_reg_list(input, des);

    //     let input = "a,b,d,x,y";
    //     let des = vec![A,B,D,X,Y];
    //     test_reg_list(input, des);
    // }
}
