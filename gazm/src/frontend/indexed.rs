use grl_sources::Position;
use unraveler::{alt, preceded, sep_pair, succeeded, tag, tuple, wrapped_cut};
use unraveler::match_span as ms;

use super::{
    get_index_reg, get_reg, get_text, parse_expr, parse_failure,
    parse_opcode_reg_pair, parse_reg_set, parse_sq_bracketed, IdentifierKind, PResult,
    TSpan, TokenKind,
    TokenKind::{Comma, Identifier, Minus, Plus},
};

use crate::{
    item::{Item, Node},
    item6809::MC6809,
    parse6809::opcodes::OPCODES_REC,
};

use crate::item6809::{
    AddrModeParseType,
    AddrModeParseType::Inherent as ParseInherent,
    IndexParseType,
    MC6809::{OpCode, Operand, OperandIndexed},
};

use emu6809::cpu::RegEnum;
use emu6809::isa::{AddrModeEnum, Instruction, InstructionInfo};

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
    let (rest, matched) = preceded(tag(Comma), succeeded(get_index_reg, Plus))(input)?;
    let index_type = IndexParseType::Plus(matched);
    Ok((rest, index_type))
}

fn get_post_inc_inc(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag(Comma), succeeded(get_index_reg, tag([Plus, Plus])))(input)?;
    let index_type = IndexParseType::PlusPlus(matched);
    Ok((rest, index_type))
}

fn get_pre_dec_dec(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag([Comma, Minus, Minus]), get_index_reg)(input)?;
    let index_type = IndexParseType::SubSub(matched);
    Ok((rest, index_type))
}

fn get_pre_dec(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(tag([Comma, Minus]), get_index_reg)(input)?;
    let index_type = IndexParseType::Sub(matched);
    Ok((rest, index_type))
}

fn get_zero(input: TSpan) -> PResult<IndexParseType> {
    let (rest, matched) = preceded(Comma, get_reg)(input)?;
    let index_type = IndexParseType::Zero(matched);
    Ok((rest, index_type))
}

fn get_add_a(input: TSpan) -> PResult<IndexParseType> {
    let reg = Identifier(IdentifierKind::Register(RegEnum::A));
    let (rest, matched) = preceded(tag([reg, Comma]), get_index_reg)(input)?;
    let index_type = IndexParseType::AddA(matched);
    Ok((rest, index_type))
}

fn get_add_b(input: TSpan) -> PResult<IndexParseType> {
    let reg = Identifier(IdentifierKind::Register(RegEnum::B));
    let (rest, matched) = preceded(tag([reg, Comma]), get_index_reg)(input)?;
    let index_type = IndexParseType::AddB(matched);
    Ok((rest, index_type))
}

fn get_add_d(input: TSpan) -> PResult<IndexParseType> {
    let reg = Identifier(IdentifierKind::Register(RegEnum::D));
    let (rest, matched) = preceded(tag([reg, Comma]), get_index_reg)(input)?;
    let index_type = IndexParseType::AddD(matched);
    Ok((rest, index_type))
}

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
#[allow(dead_code)]
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

fn parse_offset(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (expr, reg))) = ms(sep_pair(parse_expr, Comma, get_index_reg))(input)?;

    let offset = IndexParseType::ConstantOffset(reg);
    let item = MC6809::operand_from_index_mode(offset, false);
    Ok((rest, Node::from_item_kid_tspan(item, expr, sp)))
}

fn parse_pc_offset(input: TSpan) -> PResult<Node> {
    use emu6809::cpu::RegEnum::*;
    let r_kind = PC.into();
    let (rest, (sp, expr)) = ms(succeeded(parse_expr, tag([Comma, r_kind])))(input)?;
    let item = MC6809::operand_from_index_mode(IndexParseType::PCOffset, false);
    let matched = Node::from_item_kid_tspan(item, expr, sp);
    Ok((rest, matched))
}

fn parse_extended_indirect(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(parse_sq_bracketed(parse_expr))(input)?;
    let item = MC6809::operand_from_index_mode(IndexParseType::ExtendedIndirect, false);
    let matched = Node::from_item_kid_tspan(item, matched,sp);
    Ok((rest, matched))
}

fn parse_no_arg_indexed(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(get_no_arg_indexed)(input)?;
    let matched = Node::from_item_tspan(OperandIndexed(matched, false).into(), sp);
    Ok((rest, matched))
}

fn parse_no_arg_indexed_allowed_indirect(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(get_no_arg_indexed)(input)?;

    match matched {
        IndexParseType::Plus(_) => {
            let err = "Post-increment indexing not valid indirectly";
            Err(parse_failure(err, sp))
        }
        IndexParseType::Sub(_) => {
            let err = "Pre-decrement indexing not valid indirectly";
            Err(parse_failure(err, sp))
        }
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
    let mut indexed = alt((parse_no_arg_indexed, parse_pc_offset, parse_offset));
    indexed(input)
}

pub fn parse_indexed(input: TSpan) -> PResult<Node> {
    alt((
        parse_indexed_indirect,
        parse_extended_indirect,
        parse_indexed_direct,
    ))(input)
}
