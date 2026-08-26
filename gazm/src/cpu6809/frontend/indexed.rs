#![allow(unused_imports)]

use serde_yaml::Index;
use unraveler::{
    alt, and_then, cut, map, match_span as ms, pair, preceded, sep_pair, succeeded, tag_kinds,
    ParseError, Severity,
};

use crate::help::ErrCode::{self, *};
use emu6809::cpu::RegEnum;

use crate::frontend::{
    TokenKind::{Comma, Minus, Plus},
    *,
};

use super::{IndexParseType, NodeKind6809::OperandIndexed, *};

fn get_pre_dec(input: TSpan) -> PResult<IndexParseType> {
    map(preceded(Minus, cut(get_index_reg)), |r| {
        IndexParseType::PreDec(r)
    })(input)
}

fn get_pre_dec_dec(input: TSpan) -> PResult<IndexParseType> {
    map(preceded(tag_kinds([Minus, Minus]), get_index_reg), |r| {
        IndexParseType::PreDecDec(r)
    })(input)
}

fn check_index_reg<'a>(m: (TSpan<'a>, (TSpan<'a>, RegEnum))) -> PResult<'a, RegEnum> {
    let (rest, (sp, reg)) = m;
    if reg.valid_for_index() {
        Ok((rest, reg))
    } else {
        err_fatal(sp, ExpectedIndexRegister6809)
    }
}

fn get_post_inc(input: TSpan) -> PResult<IndexParseType> {
    use IndexParseType::PostInc;
    map(
        and_then(succeeded(ms(get_register), Plus), check_index_reg),
        PostInc,
    )(input)
}

fn get_post_inc_inc(input: TSpan) -> PResult<IndexParseType> {
    let postfix = tag_kinds([Plus, Plus]);

    map(
        and_then(succeeded(ms(get_register), postfix), check_index_reg),
        IndexParseType::PostIncInc,
    )(input)
}

/// Parses for ,<index reg>
fn get_zero(input: TSpan) -> PResult<IndexParseType> {
    map(cut(get_index_reg), IndexParseType::Zero)(input)
}

fn get_pc_offset(input: TSpan) -> PResult<IndexParseType> {
    map(get_this_reg(RegEnum::PC), |_| IndexParseType::PCOffset)(input)
}

/// Get indexed arg direct (not wrapped in square brackets)
fn get_indexed_direct(input: TSpan) -> PResult<IndexParseType> {
    preceded(
        Comma,
        cut(alt((
            get_pre_dec_dec,
            get_pre_dec,
            get_post_inc_inc,
            get_post_inc,
            get_pc_offset,
            get_zero,
        ))),
    )(input)
}

/// Parse for a,<ireg>, b,<ireg> or d,<ireg>
/// fatal error if wget a reg pair but not a valud abd indexed pair
fn get_abd_indexed(input: TSpan) -> PResult<IndexParseType> {
    use {IndexParseType::*, RegEnum::*};

    let (rest, (sp, abd_reg)) = succeeded(ms(get_register), Comma)(input)?;

    let abd_reg = abd_reg
        .valid_abd()
        .then_some(abd_reg)
        .ok_or(fatal(sp, ExpectedAbd6809))?;

    let (rest, idx_reg) = cut(get_index_reg)(rest)?;

    let matched = match abd_reg {
        A => AddA(idx_reg),
        B => AddB(idx_reg),
        D => AddD(idx_reg),
        _ => panic!("Internal error"),
    };

    Ok((rest, matched))
}

pub fn get_indexed(input: TSpan) -> PResult<IndexParseType> {
    alt((get_abd_indexed, get_indexed_direct))(input)
}
