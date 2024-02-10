#![deny(unused_imports)]
use crate::help::ErrCode::*;
use emu6809::cpu::RegEnum;
use std::collections::HashSet;
use unraveler::{cut, match_span as ms, sep_list, sep_pair};

use crate::frontend::{
    err_error, err_fatal, error, get_label_string, PResult, TSpan,
    Node,
    TokenKind::*,
    from_item_tspan,
};

use crate::cpu6809::{
    frontend::{
        AddrModeParseType,
        NodeKind6809::{Operand, RegisterSet},
    },
    NodeKind,
};

pub fn get_comma_sep_reg_pair(input: TSpan) -> PResult<(TSpan, RegEnum, TSpan, RegEnum)> {
    let (rest, ((sp_r1, r1), (sp_r2, r2))) =
        sep_pair(ms(get_register), Comma, ms(get_register))(input)?;
    Ok((rest, (sp_r1, r1, sp_r2, r2)))
}

pub fn parse_reg_set_operand(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(parse_reg_set)(input)?;
    let matched =
        from_item_tspan(Operand(AddrModeParseType::RegisterSet), sp)
            .with_child(matched);
    Ok((rest, matched))
}

pub fn parse_reg_set(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(get_reg_set)(input)?;
    let item = NodeKind::TargetSpecific(RegisterSet(matched).into());
    let node = from_item_tspan(item, sp);
    Ok((rest, node))
}

/// Parse opcodes with 2 reg list
/// eg tfr a,b
pub fn parse_opcode_reg_pair(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::RegisterPair;
    let (rest, (sp, (a, b))) = ms(sep_pair(get_register, Comma, cut(get_register)))(input)?;
    let node = from_item_tspan(Operand(RegisterPair(a, b)), sp);
    Ok((rest, node))
}

fn parse_this_reg_local(input: TSpan, r: RegEnum) -> PResult<RegEnum> {
    use crate::help::ErrCode;

    let (rest, (sp, matched)) = ms(get_register)(input)?;

    if matched != r {
        err_error(sp, ErrCode::ExpectedRegister6809)
    } else {
        Ok((rest, matched))
    }
}

pub fn get_this_reg(r: RegEnum) -> impl FnMut(TSpan) -> PResult<RegEnum> + Copy {
    move |i| parse_this_reg_local(i, r)
}

fn get_reg_set(input: TSpan) -> PResult<HashSet<RegEnum>> {
    use crate::help::ErrCode::*;

    // TODO : Optimisation
    // rewrite so
    // - parse for an initial register, error if not
    // - parse for a comma then a comma sep list
    // - fatal error if we can't parse a register

    let mut hash_ret = HashSet::new();
    let (rest, (sp, matched)) = ms(sep_list(get_register, Comma))(input)?;

    for r in matched {
        if hash_ret.contains(&r) {
            return err_fatal(sp, DuplicateRegisters6809);
        }
        hash_ret.insert(r);
    }

    Ok((rest, hash_ret))
}

pub fn get_index_reg(input: TSpan) -> PResult<RegEnum> {
    let (rest, (sp, matched)) =
        ms(get_register)(input).map_err(|e| e.change_kind(ExpectedIndexRegister6809))?;

    matched
        .valid_for_index()
        .then_some((rest, matched))
        .ok_or(error(sp, ExpectedIndexRegister6809))
}

/// Parse a single register
pub fn get_register(input: TSpan) -> PResult<RegEnum> {
    let (rest, (sp, text)) = ms(get_label_string)(input)?;

    text.as_str()
        .parse::<RegEnum>()
        .map(|reg| (rest, reg))
        .map_err(|_| error(sp, ExpectedRegister6809))
}
