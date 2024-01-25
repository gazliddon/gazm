#![deny(unused_imports)]
use unraveler::{alt, match_span as ms, pair, sep_pair, succeeded};

use crate::cpu6809::assembler::Assembler6809;

use crate::frontend::{
    err_fatal, parse_sq_bracketed, GazmParser, Item, Node, PResult, TSpan, TokenKind::Comma,
};

use super::{
    get_index_reg, get_this_reg, indexed::get_indexed, IndexParseType, MC6809,
    MC6809::OperandIndexed,
};

use crate::help::ErrCode;

impl GazmParser<Assembler6809> {
    /// Parses for simple offset indexed addressing
    /// ```    addr,<index reg>```
    fn parse_offset(input: TSpan) -> PResult<Node<MC6809>> {
        let (rest, (sp, (expr, reg))) =
            ms(sep_pair(Self::parse_expr, Comma, get_index_reg))(input)?;
        let offset = IndexParseType::ConstantOffset(reg);
        let item = MC6809::operand_from_index_mode(offset, false);
        Ok((rest, Self::from_item_kid_tspan(item, expr, sp)))
    }

    /// Parses for simple pc offset addressing
    /// ```    offset,pc```
    fn parse_pc_offset(input: TSpan) -> PResult<Node<MC6809>> {
        use emu6809::cpu::RegEnum::*;
        let (rest, (sp, expr)) =
            ms(succeeded(Self::parse_expr, pair(Comma, get_this_reg(PC))))(input)?;
        let item = MC6809::operand_from_index_mode(IndexParseType::PCOffset, false);
        let matched = Self::from_item_kid_tspan(item, expr, sp);
        Ok((rest, matched))
    }

    /// Parses for extended indirect
    /// ```    \[addr\]```
    fn parse_extended_indirect(input: TSpan) -> PResult<Node<MC6809>> {
        let (rest, (sp, matched)) = ms(parse_sq_bracketed(Self::parse_expr))(input)?;
        let item = MC6809::operand_from_index_mode(IndexParseType::ExtendedIndirect, false);
        let matched = Self::from_item_kid_tspan(item, matched, sp);
        Ok((rest, matched))
    }

    /// Pares for addr mode without an offset
    ///     ,y
    ///     ,-u
    fn parse_index_only(input: TSpan) -> PResult<Node<MC6809>> {
        let (rest, (sp, matched)) = ms(get_indexed)(input)?;
        let matched = Self::from_item_tspan(OperandIndexed(matched, false).into(), sp);
        Ok((rest, matched))
    }

    fn parse_no_arg_indexed_allowed_indirect(input: TSpan) -> PResult<Node<MC6809>> {
        use ErrCode::*;

        let (rest, (sp, matched)) = ms(get_indexed)(input)?;

        match matched {
            IndexParseType::PostInc(_) => err_fatal(sp, ErrIndexModeNotValidIndirect),
            IndexParseType::PreDec(_) => err_fatal(sp, ErrIndexModeNotValidIndirect),
            _ => {
                let matched = Self::from_item_tspan(OperandIndexed(matched, false).into(), sp);
                Ok((rest, matched))
            }
        }
    }

    fn parse_indexed_indirect(input: TSpan) -> PResult<Node<MC6809>> {
        let indexed_indirect = alt((
            Self::parse_no_arg_indexed_allowed_indirect,
            Self::parse_pc_offset,
            Self::parse_offset,
        ));
        let (rest, mut matched) = parse_sq_bracketed(indexed_indirect)(input)?;

        if let Item::CpuSpecific(OperandIndexed(amode, _)) = matched.item {
            matched.item = OperandIndexed(amode, true).into();
        } else {
            panic!("Should not happen")
        };

        Ok((rest, matched))
    }

    fn parse_indexed_direct(input: TSpan) -> PResult<Node<MC6809>> {
        alt((
            Self::parse_index_only,
            Self::parse_pc_offset,
            Self::parse_offset,
        ))(input)
    }

    pub fn parse_indexed(input: TSpan) -> PResult<Node<MC6809>> {
        alt((
            Self::parse_indexed_indirect,
            Self::parse_extended_indirect,
            Self::parse_indexed_direct,
        ))(input)
    }
}