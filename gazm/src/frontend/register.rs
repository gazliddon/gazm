#![deny(unused_imports)]
use crate::help::ErrCode::*;
use emu6809::cpu::RegEnum;
use std::collections::HashSet;
use unraveler::{cut, map, match_span as ms, sep_list, sep_pair};

use super::{
    err_error, err_fatal, error, get_text,
    item6809::{
        AddrModeParseType,
        MC6809::{Operand, RegisterSet},
    },
    IdentifierKind, Item, Node, PResult, TSpan,
    TokenKind::{self, *},
};

pub fn get_comma_sep_reg_pair(input: TSpan) -> PResult<(TSpan, RegEnum, TSpan, RegEnum)> {
    let (rest, ((sp_r1, r1), (sp_r2, r2))) =
        sep_pair(ms(get_register), Comma, ms(get_register))(input)?;
    Ok((rest, (sp_r1, r1, sp_r2, r2)))
}

pub fn parse_reg_set(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(get_reg_set)(input)?;
    let item = Item::Cpu(RegisterSet(matched));
    let node = Node::from_item_tspan(item, sp);
    Ok((rest, node))
}

pub fn parse_reg_set_operand(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(parse_reg_set)(input)?;
    let matched = Node::from_item_tspan(Operand(AddrModeParseType::RegisterSet).into(), sp)
        .with_child(matched);
    Ok((rest, matched))
}

fn parse_this_reg_local(input: TSpan, r: RegEnum) -> PResult<RegEnum> {
    use crate::help::ErrCode;

    let (rest, (sp, matched)) = ms(get_register)(input)?;

    if matched != r {
        err_error(sp, ErrCode::ErrExpectedRegister)
    } else {
        Ok((rest, matched))
    }
}

pub fn get_this_reg(r: RegEnum) -> impl FnMut(TSpan) -> PResult<RegEnum> + Copy {
    move |i| parse_this_reg_local(i, r)
}

fn get_reg_set(input: TSpan) -> PResult<HashSet<RegEnum>> {
    use crate::help::ErrCode::*;
    let mut hash_ret = HashSet::new();
    let (rest, (sp, matched)) = ms(sep_list(get_register, Comma))(input)?;

    for r in matched {
        if hash_ret.contains(&r) {
            return err_fatal(sp, ErrDuplicateRegisters);
        }
        hash_ret.insert(r);
    }

    Ok((rest, hash_ret))
}

pub fn get_index_reg(input: TSpan) -> PResult<RegEnum> {
    let (rest, (sp, matched)) =
        ms(get_register)(input).map_err(|e| e.change_kind(ErrExpectedIndexRegister))?;

    matched
        .valid_for_index()
        .then(|| (rest, matched))
        .ok_or(error(sp, ErrExpectedIndexRegister))
}

/// Parse a single register
pub fn get_register(input: TSpan) -> PResult<RegEnum> {
    use {IdentifierKind::*, TokenKind::*};

    let (rest, (sp, text)) = map(ms(Identifier(Label)), |(sp, _)| (sp, get_text(sp)))(input)?;

    text.as_str()
        .parse::<RegEnum>()
        .map(|reg| (rest, reg))
        .map_err(|_| error(sp, ErrExpectedRegister))
}

/// Parse opcodes with 2 reg list
/// eg tfr a,b
pub fn parse_opcode_reg_pair(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::RegisterPair;
    let (rest, (sp, (a, b))) = ms(sep_pair(get_register, Comma, cut(get_register)))(input)?;
    let node = Node::from_item_tspan(Operand(RegisterPair(a, b)).into(), sp);
    Ok((rest, node))
}
