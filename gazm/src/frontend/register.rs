#![deny(unused_imports)]
use emu6809::cpu::RegEnum;
use std::collections::HashSet;
use unraveler::{match_span as ms, sep_list, sep_pair, Severity, };

use super::{
    FrontEndError,
    FrontEndErrorKind,
    IdentifierKind,
    PResult, TSpan, 
    TokenKind::{self, *},
};

use crate::{
    item::{Item, Node},
    item6809::{AddrModeParseType, MC6809::Operand, MC6809::RegisterSet}, frontend::get_text,
};

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
    use FrontEndErrorKind::*;

    let (rest,(sp, matched )) = ms(parse_register)(input)?;

    if matched != r {
        Err(FrontEndError::new(sp,ExpectedDifferentRegister(matched, r), Severity::Error))
    } else {
        Ok((rest,matched))
    }
}

pub fn parse_this_reg(r: RegEnum) -> impl FnMut(TSpan) -> PResult<RegEnum>  + Copy{ 
    move |i| parse_this_reg_local(i,r)
}

fn get_reg_set(input: TSpan) -> PResult<HashSet<RegEnum>> {
    let mut hash_ret = HashSet::new();
    let (rest, (sp, matched)) = ms(sep_list(parse_register, Comma))(input)?;

    for r in matched {
        if hash_ret.contains(&r) {
            return Err(FrontEndError::new(sp,FrontEndErrorKind::DuplicateRegisterInRegisterSet,Severity::Fatal))
        }
        hash_ret.insert(r);
    }

    Ok((rest, hash_ret))
}

pub fn get_index_reg(input: TSpan) -> PResult<RegEnum> {
    let (rest, (sp, matched)) = ms(parse_register)(input)?;
    if matched.is_valid_for_index() {
        Ok((rest, matched))
    } else {
        Err(FrontEndError::new(sp,FrontEndErrorKind::ExpectedAnIndexRegister, Severity::Error))
    }
}



pub fn parse_register(input: TSpan) -> PResult<RegEnum> {
    use TokenKind::*;
    use IdentifierKind::*;

    let (rest,(sp,_matched)) = ms(Identifier(Label))(input)?;

    let txt = get_text(sp);

    if let Ok(reg) = txt.as_str().parse::<RegEnum>() {
        Ok((rest, reg ))
    } else {
        Err(super::FrontEndError::new(sp,super::FrontEndErrorKind::ExpectedARegister, unraveler::Severity::Error))
    }
}

pub fn parse_opcode_reg_pair(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::RegisterPair;
    let (rest, (sp, (a, b))) = ms(sep_pair(parse_register, Comma, parse_register))(input)?;
    let node = Node::from_item_tspan(Operand(RegisterPair(a, b)).into(), sp);
    Ok((rest, node))
}
