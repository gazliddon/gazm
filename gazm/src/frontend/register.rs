#![deny(unused_imports)]
use emu6809::cpu::RegEnum;
use unraveler::{match_item, sep_pair, tag, };

use super::{
    match_span as ms, IdentifierKind, PResult, TSpan, Token,
    TokenKind::{self,Identifier, Comma},
    IdentifierKind::Register,
    parse_failure,
    parse_error,
    
};

use crate::{
    item::Node,
    item6809::{AddrModeParseType, MC6809::Operand}, 
};

pub fn parse_reg_set(_input: TSpan) -> PResult<Node> {
    todo!()
}

pub fn get_index_reg(input: TSpan) -> PResult<RegEnum> {
    let (rest, (sp, matched )) = ms( get_reg )(input)?;

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
        Err(parse_error("Not a reg?",sp))
    }
}

pub fn parse_opcode_reg_pair(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::RegisterPair;
    let (rest, (sp, (a, b))) = ms(sep_pair(get_reg, Comma, get_reg))(input)?;

    let node = Node::from_item_tspan(
        Operand(RegisterPair(a, b)).into(),
        sp
    );
    Ok((rest, node))
}
