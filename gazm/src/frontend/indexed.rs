#![deny(unused_imports)]
use unraveler::{alt, cut,  pair, preceded, succeeded, tag};

use super::{
    item6809::IndexParseType,
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


/// parse for ,<index reg>+
fn get_post_inc(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag(Comma), succeeded(get_index_reg, Plus))(input)?;
    let index_type = IndexParseType::PostInc(matched);
    Ok((rest, index_type))
}

/// parse for ,<index reg>++
fn get_post_inc_inc(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag(Comma), succeeded(get_index_reg, tag([Plus, Plus])))(input)?;
    let index_type = IndexParseType::PostIncInc(matched);
    Ok((rest, index_type))
}

/// parse for ,--<index reg>
fn get_pre_dec_dec(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag([Comma, Minus, Minus]), get_index_reg)(input)?;
    let index_type = IndexParseType::PreDecDec(matched);
    Ok((rest, index_type))
}

/// parse for ,-<index reg>
fn get_pre_dec(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag([Comma, Minus]), get_index_reg)(input)?;
    let index_type = IndexParseType::PreDec(matched);
    Ok((rest, index_type))
}

/// Parses for ,<index reg>
fn get_zero(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(Comma, get_index_reg)(input)?;
    let index_type = IndexParseType::Zero(matched);
    Ok((rest, index_type))
}

/// Parses for a,<index reg>
fn get_add_a(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) =
        preceded(pair(get_this_reg(RegEnum::A), Comma), cut(get_index_reg))(input)?;
    let index_type = IndexParseType::AddA(matched);
    Ok((rest, index_type))
}

/// Parses for b,<index reg>
fn get_add_b(input: TSpan) -> PResult<IndexParseType> {
    // let reg = |i| parse_this_reg(i, RegEnum::B);
    let (rest, matched) =
        preceded(pair(get_this_reg(RegEnum::B), Comma), cut(get_index_reg))(input)?;
    let index_type = IndexParseType::AddB(matched);
    Ok((rest, index_type))
}

/// Parses for d,<index reg>
fn get_add_d(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) =
        preceded(pair(get_this_reg(RegEnum::D), Comma), cut(get_index_reg))(input)?;
    let index_type = IndexParseType::AddD(matched);
    Ok((rest, index_type))
}

/// Parses for indexed modes that do not need an offset for example
/// ```    lda ,y+```
pub fn get_no_arg_indexed(input: TSpan) -> PResult<IndexParseType> {
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

////////////////////////////////////////////////////////////////////////////////
// Parsers



