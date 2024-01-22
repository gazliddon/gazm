#![deny(unused_imports)]
use crate::{frontend::GazmParser, help::ErrCode::*};
use emu6809::cpu::RegEnum;
use std::collections::HashSet;
use unraveler::{cut, match_span as ms, sep_list, sep_pair};

use crate::frontend::{
    err_error, err_fatal, error, get_label_string, Item, Node, PResult, TSpan, TokenKind::*,
};

use crate::cpu6809::frontend::{
    AddrModeParseType,
    MC6809::{self, Operand, RegisterSet},
};

use crate::cpu6809::assembler::Compiler6809;

pub fn get_comma_sep_reg_pair(input: TSpan) -> PResult<(TSpan, RegEnum, TSpan, RegEnum)> {
    let (rest, ((sp_r1, r1), (sp_r2, r2))) =
        sep_pair(ms(get_register), Comma, ms(get_register))(input)?;
    Ok((rest, (sp_r1, r1, sp_r2, r2)))
}

impl GazmParser<Compiler6809> {
    pub fn parse_reg_set(input: TSpan) -> PResult<Node<MC6809>> {
        let (rest, (sp, matched)) = ms(get_reg_set)(input)?;
        let item = Item::CpuSpecific(RegisterSet(matched));
        let node = Self::from_item_tspan(item, sp);
        Ok((rest, node))
    }

    pub fn parse_reg_set_operand(input: TSpan) -> PResult<Node<MC6809>> {
        let (rest, (sp, matched)) = ms(Self::parse_reg_set)(input)?;
        let matched = Self::from_item_tspan(Operand(AddrModeParseType::RegisterSet).into(), sp)
            .with_child(matched);
        Ok((rest, matched))
    }
    /// Parse opcodes with 2 reg list
    /// eg tfr a,b
    pub fn parse_opcode_reg_pair(input: TSpan) -> PResult<Node<MC6809>> {
        use AddrModeParseType::RegisterPair;
        let (rest, (sp, (a, b))) = ms(sep_pair(get_register, Comma, cut(get_register)))(input)?;
        let node = Self::from_item_tspan(Operand(RegisterPair(a, b)).into(), sp);
        Ok((rest, node))
    }
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

    // TODO
    // rewrite so
    // - parse for an initial register, error if not
    // - parse for a comma then a comma sep list
    // - fatal error if we can't parse a register

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
        .then_some((rest, matched))
        .ok_or(error(sp, ErrExpectedIndexRegister))
}

/// Parse a single register
pub fn get_register(input: TSpan) -> PResult<RegEnum> {
    let (rest, (sp, text)) = ms(get_label_string)(input)?;

    text.as_str()
        .parse::<RegEnum>()
        .map(|reg| (rest, reg))
        .map_err(|_| error(sp, ErrExpectedRegister))
}
