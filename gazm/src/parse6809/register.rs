use crate::{
    error::IResult,
    item::{Item, Node},
    item6809::MC6809::RegisterSet,
    parse::locate::Span,
    parse::util,
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

    if let Item::Cpu(RegisterSet(regs)) = &matched.item {
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
#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::__Deref;
    use pretty_assertions::{assert_eq, assert_ne};
}
