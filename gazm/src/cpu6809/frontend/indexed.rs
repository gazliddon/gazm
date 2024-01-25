#![allow(unused_imports)]

use serde::de::Expected;
use serde_yaml::Index;
use tower_lsp::lsp_types::RegularExpressionsClientCapabilities;
use unraveler::{
    alt, and_then, cut, map, match_span as ms, pair, preceded, sep_pair, succeeded, tag,
    ParseError, Severity,
};

use crate::help::ErrCode;
use crate::help::ErrCode::*;
use emu6809::cpu::RegEnum;

use crate::frontend::{
    TokenKind::{Comma, Minus, Plus},
    *,
};

use super::{IndexParseType, MC6809::OperandIndexed, *};

fn get_pre_dec(input: TSpan) -> PResult<IndexParseType> {
    map(preceded(Minus, cut(get_index_reg)), |r| {
        IndexParseType::PreDec(r)
    })(input)
}

fn get_pre_dec_dec(input: TSpan) -> PResult<IndexParseType> {
    map(preceded(tag([Minus, Minus]), get_index_reg), |r| {
        IndexParseType::PreDecDec(r)
    })(input)
}

fn check_index_reg<'a>(m: (TSpan<'a>, (TSpan<'a>, RegEnum))) -> PResult<'a, RegEnum> {
    let (rest, (sp, reg)) = m;
    if reg.valid_for_index() {
        Ok((rest, reg))
    } else {
        err_fatal(sp, ErrExpectedIndexRegister)
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
    let postfix = tag([Plus, Plus]);

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

fn check_for_illegal_indirect<'a>(
    res: (TSpan<'a>, (TSpan<'a>, IndexParseType)),
) -> PResult<IndexParseType> {
    let (rest, (sp, matched)) = res;

    if matched.allowed_indirect() {
        Ok((rest, matched))
    } else {
        err_fatal(sp, ErrIndexModeNotValidIndirect)
    }
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
        .ok_or(fatal(sp, ErrExpectedAbd))?;

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

#[allow(unused_imports)]
#[allow(unused_variables)]
mod test {
    // use grl_sources::grl_symbols::deserialize;
    // use itertools::Itertools;
    // use unraveler::{all, Collection, ParseError, ParseErrorKind, Parser};

    // use super::*;
    // use crate::frontend::*;
    // use crate::opts::Opts;
    // use IndexParseType::*;
    // use RegEnum::*;

    // fn test_parse_err<P, O, E>(
    //     text: &str,
    //     desired: E,
    //     mut p: P,
    //     fatal: bool,
    // ) -> Result<(), FrontEndError>
    // where
    //     P: for<'a> Parser<TSpan<'a>, O, FrontEndError>,
    //     O: PartialEq + std::fmt::Debug,
    //     E: Into<FrontEndErrorKind>,
    // {
    //     let desired: FrontEndErrorKind = desired.into();

    //     println!("\nAbout to parse for an error {text}");

    //     let opts = Opts::default();
    //     let source_file = create_source_file(text);
    //     let tokens = to_tokens_no_comment(&source_file);
    //     let toke_kinds = tokens.iter().map(|t| t.kind).collect_vec();
    //     println!("TOKES: {:?}", toke_kinds);

    //     let span = make_tspan(&tokens, &source_file, &opts);

    //     let res = p.parse(span);

    //     match res {
    //         Ok(_) => panic!("Parser did not fail!"),
    //         Err(e) => {
    //             assert_eq!(e.kind, desired);
    //             assert_eq!(e.is_fatal(), fatal);
    //             Ok(())
    //         }
    //     }
    // }

    // fn test_parse_all<P, O>(text: &str, desired: O, p: P) -> Result<(), FrontEndError>
    // where
    //     P: for<'a> Parser<TSpan<'a>, O, FrontEndError>,
    //     O: std::fmt::Debug,
    //     O: PartialEq,
    // {
    //     println!("\nAbout to parse {text}");

    //     let opts = Opts::default();
    //     let source_file = create_source_file(text);
    //     let tokens = to_tokens_no_comment(&source_file);
    //     let toke_kinds = tokens.iter().map(|t| t.kind).collect_vec();
    //     println!("TOKES: {:?}", toke_kinds);

    //     let span = make_tspan(&tokens, &source_file, &opts);

    //     let (rest, matched) = all(p)(span)?;
    //     assert_eq!(matched, desired);
    //     Ok(())
    // }

    // fn test_parse<P, O>(text: &str, desired: O, mut p: P) -> Result<(), FrontEndError>
    // where
    //     P: for<'a> Parser<TSpan<'a>, O, FrontEndError>,
    //     O: std::fmt::Debug,
    //     O: PartialEq,
    // {
    //     println!("\nAbout to parse {text}");

    //     let opts = Opts::default();
    //     let source_file = create_source_file(text);
    //     let tokens = to_tokens_no_comment(&source_file);
    //     let toke_kinds = tokens.iter().map(|t| t.kind).collect_vec();
    //     println!("TOKES: {:?}", toke_kinds);

    //     let span = make_tspan(&tokens, &source_file, &opts);

    //     let (rest, matched) = p.parse(span)?;
    //     assert_eq!(matched, desired);
    //     Ok(())
    // }

    // #[test]
    // fn test_abd() {
    //     test_parse("a,u", AddA(U), get_abd_indexed).expect("add a");
    //     test_parse("b,u", AddB(U), get_abd_indexed).expect("add b");
    //     test_parse("d,u", AddD(U), get_abd_indexed).expect("add d");

    //     test_parse_err("x,u", ErrExpectedAbd, get_abd_indexed, true).unwrap();
    //     test_parse_err("a,a", ErrExpectedIndexRegister, get_abd_indexed, true).unwrap();
    // }

    // #[test]
    // fn test_individual() {
    //     test_parse_all("y", Y, get_index_reg).expect("Index register");
    //     test_parse_all("-y", PreDec(Y), get_pre_dec).expect("Pre dec");
    //     test_parse("--y", PreDecDec(Y), get_pre_dec_dec).expect("Pre dec dec");
    //     test_parse("x", X, get_index_reg).expect("Index register");
    //     test_parse("x+", PostInc(X), get_post_inc).expect("Post inc ");
    //     test_parse("x++", PostIncInc(X), get_post_inc_inc).expect("Post inc");
    //     test_parse("pc", PCOffset, get_pc_offset).expect("Pc offset");
    // }

    // #[test]
    // fn test_indexed_direct() {
    //     let opts = Opts::default();

    //     let to_text = vec![
    //         (",--y", PreDecDec(Y)),
    //         (",-y", PreDec(Y)),
    //         (",y+", PostInc(Y)),
    //         (",y++", PostIncInc(Y)),
    //         (",x", Zero(X)),
    //         (",pc", PCOffset),
    //     ];

    //     for (text, desired) in to_text.into_iter() {
    //         test_parse_all(text, desired, get_indexed_direct).expect("Parsing")
    //     }
    // }
    // #[test]
    // fn test_errors() {
    //     test_parse_err(",-a", ErrExpectedIndexRegister, get_indexed_direct, true).unwrap();
    //     test_parse_err(",,,", ErrExpectedIndexRegister, get_indexed_direct, true).unwrap();
    //     test_parse_err("q", ErrExpectedRegister, get_register, false).unwrap();
    // }
}
