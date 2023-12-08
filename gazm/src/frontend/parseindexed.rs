#![deny(unused_imports)]
use unraveler::{alt, match_span as ms, pair, sep_pair, succeeded};

use super::{
    item6809::MC6809,
    item6809::{IndexParseType, MC6809::OperandIndexed},
    indexed::get_indexed,
    TokenKind::Comma,
    *,
};

use crate::help::ErrCode;

/// Parses for simple offset indexed addressing
/// ```    addr,<index reg>```
fn parse_offset(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (expr, reg))) = ms(sep_pair(parse_expr, Comma, get_index_reg))(input)?;
    let offset = IndexParseType::ConstantOffset(reg);
    let item = MC6809::operand_from_index_mode(offset, false);
    Ok((rest, Node::from_item_kid_tspan(item, expr, sp)))
}

/// Parses for simple pc offset addressing
/// ```    offset,pc```
fn parse_pc_offset(input: TSpan) -> PResult<Node> {
    use emu6809::cpu::RegEnum::*;
    let (rest, (sp, expr)) = ms(succeeded(parse_expr, pair(Comma, get_this_reg(PC))))(input)?;
    let item = MC6809::operand_from_index_mode(IndexParseType::PCOffset, false);
    let matched = Node::from_item_kid_tspan(item, expr, sp);
    Ok((rest, matched))
}

/// Parses for extended indirect
/// ```    \[addr\]```
fn parse_extended_indirect(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(parse_sq_bracketed(parse_expr))(input)?;
    let item = MC6809::operand_from_index_mode(IndexParseType::ExtendedIndirect, false);
    let matched = Node::from_item_kid_tspan(item, matched, sp);
    Ok((rest, matched))
}

/// Pares for addr mode without an offset
///     ,y
///     ,-u
fn parse_index_only(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(get_indexed)(input)?;
    let matched = Node::from_item_tspan(OperandIndexed(matched, false).into(), sp);
    Ok((rest, matched))
}

fn parse_no_arg_indexed_allowed_indirect(input: TSpan) -> PResult<Node> {
    use ErrCode::*;

    let (rest, (sp, matched)) =  ms(get_indexed)(input)?;

    match matched {
        IndexParseType::PostInc(_) => err_fatal(sp, ErrIndexModeNotValidIndirect),
        IndexParseType::PreDec(_) => err_fatal(sp, ErrIndexModeNotValidIndirect),
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
    alt((parse_index_only, parse_pc_offset, parse_offset))(input)
}

pub fn parse_indexed(input: TSpan) -> PResult<Node> {
    alt((
        parse_indexed_indirect,
        parse_extended_indirect,
        parse_indexed_direct,
    ))(input)
}
