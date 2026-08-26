#![deny(unused_imports)]
use crate::cpu_z80::assembler::ISA_DBASE;
use crate::cpukind::CpuKind;
use crate::frontend::{
    ascii_lowercase, err_fatal, from_item_children_tspan, from_item_tspan, get_text, parse_expr,
    Node, PResult, TSpan, Token, TokenKind,
};

use emuz80::isa::{Instruction, InstructionInfo};
use unraveler::{alt, kind, match_item, match_span as ms, Collection};

use super::{NodeKindZ80::OpCode, OperandParseType, Pair, Reg8};

/// One bound operand: a register/pair/bit/vector value or an expression
/// child (n/nn/d/e, or the displacement inside `(IX+d)`).
enum Part {
    Reg(Reg8),
    Pair(Pair),
    Bit(u8),
    Restart(u8),
    Expr(Node),
    IndexedExpr(Node),
}

/// How specific a template is: more literal parts win, then templates with
/// an `(IX+d)`/`(IY+d)` index (so `A,(IX+d)` beats `A,(nn)` — the latter
/// would swallow `ix-1` as an expression), then more parts, then fewer
/// expression parts. This ordering resolves every real ambiguity (e.g.
/// `LD A,B` must match `r1,r2`, not `r,n` with B as a label expression).
fn template_priority(template: &str) -> (usize, usize, usize, i32) {
    let parts: Vec<&str> = template.split(',').collect();
    let literal = parts.iter().filter(|p| !is_expr_part(p)).count();
    let indexed = if parts.iter().any(|p| *p == "(IX+d)" || *p == "(IY+d)") {
        1
    } else {
        0
    };
    let exprs = parts.iter().filter(|p| is_expr_part(p)).count();
    (literal, indexed, parts.len(), -(exprs as i32))
}

fn is_expr_part(part: &str) -> bool {
    matches!(
        part,
        "n" | "nn" | "d" | "(n)" | "(nn)" | "(IX+d)" | "(IY+d)"
    )
}

/// Match one template part against the input. Returns the bound part (if
/// any) and the remaining input.
fn match_part<'a>(part: &str, input: TSpan<'a>) -> Option<(Option<Part>, TSpan<'a>)> {
    let ident = |input: TSpan<'a>| -> Option<(String, TSpan<'a>)> {
        let (rest, sp) =
            kind::<TSpan<'a>, crate::frontend::FrontEndError>(TokenKind::Identifier)(input).ok()?;
        Some((ascii_lowercase(&get_text(sp)).into_owned(), rest))
    };
    let number = |input: TSpan<'a>| -> Option<(i64, TSpan<'a>)> {
        let (rest, (sp, _)) =
            ms::<_, TSpan<'a>, _, crate::frontend::FrontEndError>(match_item(|t: &Token| {
                matches!(t.kind, TokenKind::Number(..))
            }))(input)
            .ok()?;
        let tok = sp.first()?;
        let TokenKind::Number((n, _)) = tok.kind else {
            return None;
        };
        Some((n, rest))
    };

    match part {
        "r" | "r1" | "r2" => {
            let (text, rest) = ident(input)?;
            Some((Some(Part::Reg(Reg8::from_text(&text)?)), rest))
        }
        "dd" => {
            let (text, rest) = ident(input)?;
            Some((Some(Part::Pair(Pair::from_text(&text)?)), rest))
        }
        "b" => {
            let (n, rest) = number(input)?;
            if !(0..=7).contains(&n) {
                return None;
            }
            Some((Some(Part::Bit(n as u8)), rest))
        }
        "p" => {
            // RST accepts the vector number (0-7) or the vector address
            // (8, 16, ..., 56); normalize to the vector number.
            let (n, rest) = number(input)?;
            let p = if (0..=7).contains(&n) {
                n
            } else if n > 0 && n <= 56 && n % 8 == 0 {
                n / 8
            } else {
                return None;
            };
            Some((Some(Part::Restart(p as u8)), rest))
        }
        "n" | "nn" | "d" => {
            let (rest, node) = parse_expr(input).ok()?;
            Some((Some(Part::Expr(node)), rest))
        }
        "0" | "1" | "2" => {
            let (n, rest) = number(input)?;
            (n.to_string() == part).then_some((None, rest))
        }
        "AF'" => {
            let (text, rest) = ident(input)?;
            if text != "af" {
                return None;
            }
            let (rest, _) =
                kind::<TSpan<'a>, crate::frontend::FrontEndError>(TokenKind::Apostrophe)(rest)
                    .ok()?;
            Some((None, rest))
        }
        "A" | "B" | "C" | "D" | "E" | "H" | "L" | "I" | "R" | "AF" | "BC" | "DE" | "HL" | "SP"
        | "IX" | "IY" | "Z" | "NZ" | "NC" | "PO" | "PE" | "P" | "M" => {
            let (text, rest) = ident(input)?;
            (text == part.to_ascii_lowercase()).then_some((None, rest))
        }
        "(HL)" | "(BC)" | "(DE)" | "(C)" | "(SP)" | "(IX)" | "(IY)" => {
            let (rest, _) =
                kind::<TSpan<'a>, crate::frontend::FrontEndError>(TokenKind::OpenBracket)(input)
                    .ok()?;
            let inner = part.trim_start_matches('(').trim_end_matches(')');
            let (text, rest) = ident(rest)?;
            if text != inner.to_ascii_lowercase() {
                return None;
            }
            let (rest, _) =
                kind::<TSpan<'a>, crate::frontend::FrontEndError>(TokenKind::CloseBracket)(rest)
                    .ok()?;
            Some((None, rest))
        }
        "(n)" | "(nn)" => {
            let (rest, _) =
                kind::<TSpan<'a>, crate::frontend::FrontEndError>(TokenKind::OpenBracket)(input)
                    .ok()?;
            let (rest, node) = parse_expr(rest).ok()?;
            let (rest, _) =
                kind::<TSpan<'a>, crate::frontend::FrontEndError>(TokenKind::CloseBracket)(rest)
                    .ok()?;
            Some((Some(Part::Expr(node)), rest))
        }
        "(IX+d)" | "(IY+d)" => {
            let (rest, _) =
                kind::<TSpan<'a>, crate::frontend::FrontEndError>(TokenKind::OpenBracket)(input)
                    .ok()?;
            let inner = part.trim_start_matches('(').trim_end_matches("+d)");
            let (text, rest) = ident(rest)?;
            if text != inner.to_ascii_lowercase() {
                return None;
            }
            // Optional signed displacement; absent means +0.
            let (rest, node) = match rest.first().map(|t| t.kind) {
                Some(TokenKind::Plus) => {
                    let (rest, _) =
                        kind::<TSpan<'a>, crate::frontend::FrontEndError>(TokenKind::Plus)(rest)
                            .ok()?;
                    parse_expr(rest).ok()?
                }
                _ => parse_expr(rest).ok()?,
            };
            let (rest, _) =
                kind::<TSpan<'a>, crate::frontend::FrontEndError>(TokenKind::CloseBracket)(rest)
                    .ok()?;
            Some((Some(Part::IndexedExpr(node)), rest))
        }
        _ => None,
    }
}

/// Build the AST operand type from the bound parts.
fn operand_type(parts: &[Part]) -> Option<OperandParseType> {
    use Part::{
        Bit, Expr as PExpr, IndexedExpr as PIndexed, Pair as PPair, Reg as PReg,
        Restart as PRestart,
    };
    Some(match parts {
        [] => OperandParseType::None,
        [PExpr(_)] | [PIndexed(_)] => OperandParseType::Expr,
        [PExpr(_), PExpr(_)] | [PIndexed(_), PExpr(_)] | [PExpr(_), PIndexed(_)] => {
            OperandParseType::ExprExpr
        }
        [PReg(r)] => OperandParseType::Reg(*r),
        [PReg(r), PExpr(_)] => OperandParseType::RegExpr(*r),
        [PReg(r), PIndexed(_)] => OperandParseType::RegIndexed(*r),
        [PIndexed(_), PReg(r)] => OperandParseType::IndexedReg(*r),
        [PReg(a), PReg(b)] => OperandParseType::RegReg(*a, *b),
        [PPair(p)] => OperandParseType::Pair(*p),
        [PPair(p), PExpr(_)] => OperandParseType::PairExpr(*p),
        [Bit(b)] => OperandParseType::BitIndirect(*b),
        [Bit(b), PIndexed(_)] => OperandParseType::BitIndexed(*b),
        [Bit(b), PReg(r)] => OperandParseType::BitReg(*b, *r),
        [Bit(b), PIndexed(_), PReg(r)] => OperandParseType::BitIndexedReg(*b, *r),
        [PRestart(p)] => OperandParseType::Restart(*p),
        _ => return None,
    })
}

fn children(parts: &[Part]) -> Vec<Node> {
    parts
        .iter()
        .filter_map(|p| match p {
            Part::Expr(n) | Part::IndexedExpr(n) => Some(n.clone()),
            _ => None,
        })
        .collect()
}

/// Pick the instruction row for a template: register forms have plain,
/// DD, and FD variants (`LD r1,r2` -> 0x40/0xDD40/0xFD40); IXH/IXL need
/// the DD row, IYH/IYL the FD row, everything else the plain row.
fn pick_row<'a>(rows: &'a [Instruction], parts: &[Part]) -> Option<&'a Instruction> {
    let want = parts.iter().find_map(|p| match p {
        Part::Reg(r) => r.prefix(),
        _ => None,
    });
    match want {
        None => rows
            .iter()
            .find(|r| r.opcode <= 0xff)
            .or_else(|| rows.first()),
        Some(0xdd00) => rows
            .iter()
            .find(|r| (r.opcode >> 8) == 0xdd)
            .or_else(|| rows.first()),
        Some(0xfd00) => rows
            .iter()
            .find(|r| (r.opcode >> 8) == 0xfd)
            .or_else(|| rows.first()),
        Some(_) => rows.first(),
    }
}

fn get_opcode(input: TSpan<'_>) -> PResult<'_, (TSpan<'_>, &'static InstructionInfo)> {
    use TokenKind::{CpuOpcode, Identifier};
    let (rest, (sp, matched)) = ms(alt((CpuOpcode(CpuKind::CpuZ80), Identifier)))(input)?;
    let text = ascii_lowercase(&get_text(matched)).into_owned();
    let info = ISA_DBASE
        .get_info(&text.to_ascii_uppercase())
        .ok_or_else(|| {
            crate::frontend::FrontEndError::error(
                input,
                crate::frontend::CpuAssemblyErrorKind::UnknownOpcode("Z80"),
            )
        })?;
    Ok((rest, (sp, info)))
}

pub fn parse_opcode(input: TSpan) -> PResult<Node> {
    let (rest, (sp, info)) = get_opcode(input)?;
    if rest.is_empty() {
        // Inherent forms (NOP, LDIR, RET, ...).
        let row = info.get("").and_then(|rows| rows.first()).ok_or_else(|| {
            crate::frontend::FrontEndError::error(
                sp,
                crate::frontend::CpuAssemblyErrorKind::MissingOperand,
            )
        })?;
        let item = OpCode(row.id(), OperandParseType::None);
        let node = from_item_tspan(item, sp);
        return Ok((rest, node));
    }

    // Try the mnemonic's templates, most specific first; ties break on the
    // template string so the parse is deterministic (HashMap order is not).
    let mut candidates: Vec<&String> = info.templates.keys().filter(|t| !t.is_empty()).collect();
    candidates.sort_by(|a, b| {
        template_priority(b)
            .cmp(&template_priority(a))
            .then_with(|| a.cmp(b))
    });

    for template in candidates {
        let parts: Vec<&str> = template.split(',').collect();
        let mut cur = rest;
        let mut bound: Vec<Part> = Vec::new();
        let mut matched = true;
        for (i, part) in parts.iter().enumerate() {
            // Templates split on ','; the input has comma tokens between
            // the operand parts.
            if i > 0 {
                if let Ok((after, _)) =
                    kind::<TSpan<'_>, crate::frontend::FrontEndError>(TokenKind::Comma)(cur)
                {
                    cur = after;
                } else {
                    matched = false;
                    break;
                }
            }
            let Some((bound_part, after)) = match_part(part, cur) else {
                matched = false;
                break;
            };
            if let Some(p) = bound_part {
                bound.push(p);
            }
            cur = after;
        }
        if !matched || !cur.is_empty() {
            continue;
        }

        let Some(op_type) = operand_type(&bound) else {
            continue;
        };
        let Some(row) = pick_row(info.get(template).expect("template from keys"), &bound) else {
            continue;
        };
        let item = OpCode(row.id(), op_type);
        let node = from_item_children_tspan(item, &children(&bound), sp);
        return Ok((cur, node));
    }

    err_fatal(sp, crate::frontend::CpuAssemblyErrorKind::OperandsDontMatch)
}

pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, opcode) = parse_opcode(input)?;
    Ok((rest, vec![opcode]))
}
