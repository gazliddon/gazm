#![deny(unused_imports)]
use emu6809::cpu::RegEnum;
use std::collections::HashSet;
use unraveler::{match_item, match_span as ms, sep_list, sep_pair, tag};

use super::{
    parse_error, parse_failure, IdentifierKind,
    IdentifierKind::Register,
    PResult, TSpan, Token,
    TokenKind::{self, *},
};

use crate::{
    item::{Item, Node},
    item6809::{AddrModeParseType, MC6809::Operand, MC6809::RegisterSet},
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

fn get_reg_set(input: TSpan) -> PResult<HashSet<RegEnum>> {
    let mut hash_ret = HashSet::new();
    let (rest, (sp, matched)) = ms(sep_list(get_reg, Comma))(input)?;

    for r in matched {
        if hash_ret.contains(&r) {
            return Err(parse_error("Duplicate registers in register set", sp));
        }
        hash_ret.insert(r);
    }

    Ok((rest, hash_ret))
}

pub fn get_index_reg(input: TSpan) -> PResult<RegEnum> {
    let (rest, (sp, matched)) = ms(get_reg)(input)?;

    if matched.is_valid_for_index() {
        Ok((rest, matched))
    } else {
        Err(parse_failure("This register is not an index register ", sp))
    }
}

pub fn reg_predicate(t: &Token) -> bool {
    matches!(&t.kind, Identifier(Register(..)))
}

pub fn get_reg_token(input: TSpan, r: RegEnum) -> PResult<RegEnum> {
    let (rest, _) = tag(TokenKind::from(r))(input)?;
    Ok((rest, r))
}

pub fn get_reg(input: TSpan) -> PResult<RegEnum> {
    let (rest, (sp, matched)) = ms(match_item(reg_predicate))(input)?;

    if let Identifier(IdentifierKind::Register(r)) = matched.kind {
        Ok((rest, r))
    } else {
        Err(parse_error("Not a reg?", sp))
    }
}

pub fn parse_opcode_reg_pair(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::RegisterPair;
    let (rest, (sp, (a, b))) = ms(sep_pair(get_reg, Comma, get_reg))(input)?;

    let node = Node::from_item_tspan(Operand(RegisterPair(a, b)).into(), sp);
    Ok((rest, node))
}
