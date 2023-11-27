#![deny(unused_imports)]
use unraveler::{alt, cut, match_span as ms, pair, preceded, sep_pair, succeeded, tag};

use super::{
    item6809::MC6809,
    item6809::{IndexParseType, MC6809::OperandIndexed},
    TokenKind::{Comma, Minus, Plus},
    *,
};

use emu6809::cpu::RegEnum;

// Addr Modes and Parsing Order
//  ,--R    SubSub(RegEnum)         parse_pre_dec_dec
//  ,-R     Sub(RegEnum)            parse_pre_dec       *NOT ALLOWED INDIRECT*
//  ,R++    PlusPlus(RegEnum)       parse_post_inc
//  ,R+     Plus(RegEnum)           parse_post_inc
//  ,R      Zero(RegEnum)           parse_zero          *NOT ALLOWED INDIRECT*
// A,R      AddA(RegEnum)           parse_add_a
// B,R      AddB(RegEnum)           parse_add_b
// D,R      AddD(RegEnum)           parse_add_d
// n,PC     PCOffset                parse_pc_offset
// n,R      ConstantOffset(RegEnum) parse_offset

// Post inc / dec
fn get_post_inc(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag(Comma), succeeded(fatal_get_index_reg, Plus))(input)?;
    let index_type = IndexParseType::Plus(matched);
    Ok((rest, index_type))
}

fn get_post_inc_inc(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag(Comma), succeeded(fatal_get_index_reg, tag([Plus, Plus])))(input)?;
    let index_type = IndexParseType::PlusPlus(matched);
    Ok((rest, index_type))
}

fn get_pre_dec_dec(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag([Comma, Minus, Minus]), fatal_get_index_reg)(input)?;
    let index_type = IndexParseType::SubSub(matched);
    Ok((rest, index_type))
}

fn get_pre_dec(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag([Comma, Minus]), fatal_get_index_reg)(input)?;
    let index_type = IndexParseType::Sub(matched);
    Ok((rest, index_type))
}

/// Parses for ```lda ,y```
fn get_zero(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(Comma, fatal_get_index_reg)(input)?;
    let index_type = IndexParseType::Zero(matched);
    Ok((rest, index_type))
}

/// Parses for ```lda a,y```
fn get_add_a(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(pair(parse_this_reg(RegEnum::A), Comma), fatal_get_index_reg)(input)?;
    let index_type = IndexParseType::AddA(matched);
    Ok((rest, index_type))
}

/// Parses for ```lda b,y```
fn get_add_b(input: TSpan) -> PResult<IndexParseType> {
    // let reg = |i| parse_this_reg(i, RegEnum::B);
    let (rest, matched) = preceded(pair(parse_this_reg(RegEnum::B), Comma), fatal_get_index_reg )(input)?;
    let index_type = IndexParseType::AddB(matched);
    Ok((rest, index_type))
}

/// Parses for ```lda d,y```
fn get_add_d(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(pair(parse_this_reg(RegEnum::D), Comma), cut( fatal_get_index_reg ))(input)?;
    let index_type = IndexParseType::AddD(matched);
    Ok((rest, index_type))
}

/// Parses for indexed modes that do not need an offset for example
/// ```    lda ,y+```
fn get_no_arg_indexed(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = alt((
        get_pre_dec_dec,
        get_post_inc_inc,
        get_pre_dec,
        get_post_inc,
        get_zero,
        get_add_a,
        get_add_b,
        get_add_d,
    ))(input)?;
    Ok((rest, matched))
}

/// Parses for indexed modes that do not need an offset and can be be indirect
/// ```    lda [,y++]```
fn get_no_arg_indexed_allowed_indirect(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = alt((
        get_pre_dec_dec,
        get_post_inc_inc,
        get_zero,
        get_add_a,
        get_add_b,
        get_add_d,
    ))(input)?;
    Ok((rest, matched))
}

/// Parses for simple offset indexed addressing
/// ```    lda addr,x```
fn parse_offset(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (expr, reg))) = ms(sep_pair(parse_expr, Comma, fatal_get_index_reg))(input)?;
    let offset = IndexParseType::ConstantOffset(reg);
    let item = MC6809::operand_from_index_mode(offset, false);
    Ok((rest, Node::from_item_kid_tspan(item, expr, sp)))
}

fn parse_pc_offset(input: TSpan) -> PResult<Node> {
    use emu6809::cpu::RegEnum::*;
    let (rest, (sp, expr)) = ms(succeeded(parse_expr, pair(Comma, parse_this_reg(PC))))(input)?;
    let item = MC6809::operand_from_index_mode(IndexParseType::PCOffset, false);
    let matched = Node::from_item_kid_tspan(item, expr, sp);
    Ok((rest, matched))
}

fn parse_extended_indirect(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(parse_sq_bracketed(parse_expr))(input)?;
    let item = MC6809::operand_from_index_mode(IndexParseType::ExtendedIndirect, false);
    let matched = Node::from_item_kid_tspan(item, matched, sp);
    Ok((rest, matched))
}

fn parse_no_arg_indexed(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(get_no_arg_indexed)(input)?;
    let matched = Node::from_item_tspan(OperandIndexed(matched, false).into(), sp);
    Ok((rest, matched))
}

fn parse_no_arg_indexed_allowed_indirect(input: TSpan) -> PResult<Node> {
    use crate::help::ErrCode;
    let (rest, (sp, matched)) = ms(get_no_arg_indexed)(input)?;

    match matched {
        IndexParseType::Plus(_) => fatal(sp, ErrCode::ErrIndexModeNotValidIndirect),
        IndexParseType::Sub(_) => fatal(sp, ErrCode::ErrIndexModeNotValidIndirect),
        _ => {
            let matched = Node::from_item_tspan(OperandIndexed(matched, false).into(), sp);
            Ok((rest, matched))
        }
    }
}

fn parse_indexed_indirect(input: TSpan) -> PResult<Node> {
    let indexed_indirect = alt((
        parse_no_arg_indexed_allowed_indirect,
        parse_pc_offset,
        parse_offset,
    ));
    let (rest, mut matched) = parse_sq_bracketed(indexed_indirect)(input)?;

    if let Item::Cpu(OperandIndexed(amode, _)) = matched.item {
        matched.item = OperandIndexed(amode, true).into();
    } else {
        panic!("Should not happen")
    };

    Ok((rest, matched))
}

fn parse_indexed_direct(input: TSpan) -> PResult<Node> {
    alt((parse_no_arg_indexed, parse_pc_offset, parse_offset))(input)
}

pub fn parse_indexed(input: TSpan) -> PResult<Node> {
    alt((
        parse_indexed_indirect,
        parse_extended_indirect,
        parse_indexed_direct,
    ))(input)
}
