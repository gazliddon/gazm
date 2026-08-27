#![deny(unused_imports)]
use crate::frontend::{
    err_fatal, fatal, from_item_children_tspan, from_item_tspan, get_label_string, get_text,
    parse_expr, AstNodeKind, CpuAssemblyErrorKind, CpuSpecific, FrontEndError, Node, PResult,
    TSpan, TokenKind,
};

use crate::cpu68000::isa::table;
use crate::cpukind::CpuKind::Cpu68000 as Cpu;

use unraveler::{alt, kind, match_item, match_span as ms, preceded, Collection, Parser};

use emu68000::isa::{Form, Size};

use crate::frontend::LabelDefinition;

use super::{Ea68000, Instr68000, MovemList, NodeKind68000, Operand68000};

use NodeKind68000::OpCode;

// ---------------------------------------------------------------------------
// mnemonic + size suffix

fn suffix_size(text: &str) -> Option<Size> {
    match text.to_ascii_lowercase().as_str() {
        ".b" => Some(Size::B),
        ".w" => Some(Size::W),
        ".l" => Some(Size::L),
        // `.s` is MadMac's explicit short branch spelling.
        ".s" => Some(Size::B),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// registers

fn reg_num(input: TSpan, prefix: char) -> Option<(u8, TSpan)> {
    let (rest, (sp, _)) = ms(get_label_string)(input).ok()?;
    let t = get_text(sp).to_ascii_uppercase();
    if t.starts_with(prefix) {
        if let Ok(n) = t[1..].parse::<u8>() {
            if n < 8 {
                return Some((n, rest));
            }
        }
    }
    None
}

fn parse_dn(input: TSpan) -> Option<(u8, TSpan)> {
    reg_num(input, 'D')
}
fn parse_an(input: TSpan) -> Option<(u8, TSpan)> {
    if let Some((n, rest)) = reg_num(input, 'A') {
        return Some((n, rest));
    }
    // `sp` is MadMac's alias for A7.
    let (rest, (sp, _)) = ms(get_label_string)(input).ok()?;
    if get_text(sp).eq_ignore_ascii_case("sp") {
        return Some((7, rest));
    }
    None
}

/// `Dn` / `An`
fn parse_reg(input: TSpan) -> Option<(Ea68000, TSpan)> {
    if let Some((n, rest)) = parse_dn(input) {
        return Some((Ea68000::Dn(n), rest));
    }
    if let Some((n, rest)) = parse_an(input) {
        return Some((Ea68000::An(n), rest));
    }
    None
}

/// index register: `Dn`/`An` with optional `.w`/`.l` and `*2/*4/*8`
fn parse_index(input: TSpan) -> Option<((u8, bool, u8, bool), TSpan)> {
    let (n, is_a, rest) = if let Some((n, rest)) = parse_dn(input) {
        (n, false, rest)
    } else if let Some((n, rest)) = parse_an(input) {
        (n, true, rest)
    } else {
        return None;
    };
    let (rest, long) = match rest.first().map(|t| t.kind) {
        Some(TokenKind::Identifier) => {
            let (rest, (_, txt)) = ms(get_label_string)(rest).ok()?;
            match suffix_size(txt.as_ref()) {
                Some(Size::L) => (rest, true),
                Some(_) => (rest, false),
                None => return None,
            }
        }
        _ => (rest, false),
    };
    let (rest, scale) = if rest.first().is_some_and(|t| t.kind == TokenKind::Star) {
        let (rest, _) = TokenKind::Star.parse(rest).ok()?;
        let (rest, (_, tok)) =
            ms::<_, TSpan<'_>, _, FrontEndError>(match_item(|t: &crate::frontend::Token<'_>| {
                matches!(t.kind, TokenKind::Number(..))
            }))(rest)
            .ok()?;
        let n = match tok.kind {
            TokenKind::Number((n, _)) => n,
            _ => return None,
        };
        match n {
            1 => (rest, 0),
            2 => (rest, 1),
            4 => (rest, 2),
            8 => (rest, 3),
            _ => return None,
        }
    } else {
        (rest, 0)
    };
    Some(((n, is_a, scale, long), rest))
}

/// `(An)` / `(An)+` / `-(An)`
fn parse_paren_reg(input: TSpan) -> Option<(Ea68000, TSpan)> {
    if input.first().is_some_and(|t| t.kind == TokenKind::Minus) {
        let (rest, _) = TokenKind::Minus.parse(input).ok()?;
        let (rest, _) = TokenKind::OpenBracket.parse(rest).ok()?;
        let (n, rest) = parse_an(rest)?;
        let (rest, _) = TokenKind::CloseBracket.parse(rest).ok()?;
        return Some((Ea68000::AnPreDec(n), rest));
    }
    let (rest, _) = TokenKind::OpenBracket.parse(input).ok()?;
    let (n, rest) = parse_an(rest)?;
    let (rest, _) = TokenKind::CloseBracket.parse(rest).ok()?;
    if rest.first().is_some_and(|t| t.kind == TokenKind::Plus) {
        let (rest, _) = TokenKind::Plus.parse(rest).ok()?;
        return Some((Ea68000::AnPostInc(n), rest));
    }
    Some((Ea68000::AnInd(n), rest))
}

/// The `.w`/`.l` suffix token after an expression (a dot-label identifier).
fn parse_size_suffix(input: TSpan) -> Option<(Size, TSpan)> {
    let (rest, (_, txt)) = ms(get_label_string)(input).ok()?;
    suffix_size(txt.as_ref()).map(|s| (s, rest))
}

// ---------------------------------------------------------------------------
// expression-bearing forms

/// `expr(An[,Xn])` / `expr(PC[,Xn])`. A bare-identifier base is built
/// manually so the expression parser's function-call form can't swallow
/// the parens.
fn parse_disp_paren(input: TSpan) -> Option<(Ea68000, TSpan, Node)> {
    let (rest, expr): (TSpan, Node) = if input
        .first()
        .is_some_and(|t| matches!(t.kind, TokenKind::Identifier | TokenKind::Label))
    {
        let (rest, sp) = kind::<TSpan<'_>, FrontEndError>(TokenKind::Identifier)(input).ok()?;
        if !rest
            .first()
            .is_some_and(|t| t.kind == TokenKind::OpenBracket)
        {
            return None;
        }
        let base = get_text(sp).to_string();
        (
            rest,
            from_item_tspan(AstNodeKind::Label(LabelDefinition::Text(base)), sp),
        )
    } else {
        let (rest, (_, node)) = ms(parse_expr)(input).ok()?;
        (rest, node)
    };
    let (rest, _) = TokenKind::OpenBracket.parse(rest).ok()?;

    // base register: An or PC
    let (is_pc, an, rest) = match parse_an(rest) {
        Some((an, rest)) => (false, an, rest),
        None => {
            let (rest, (_, txt)) = ms(get_label_string)(rest).ok()?;
            if txt.eq_ignore_ascii_case("pc") {
                (true, 0, rest)
            } else {
                return None;
            }
        }
    };

    let (rest, idx) = if rest.first().is_some_and(|t| t.kind == TokenKind::Comma) {
        let (rest, _) = TokenKind::Comma.parse(rest).ok()?;
        let (idx, rest) = parse_index(rest)?;
        (rest, Some(idx))
    } else {
        (rest, None)
    };
    let (rest, _) = TokenKind::CloseBracket.parse(rest).ok()?;

    let ea = match (is_pc, idx) {
        (true, Some((idx, is_a, scale, long))) => Ea68000::PcDispIdx {
            idx,
            is_a,
            scale,
            long,
        },
        (true, None) => Ea68000::PcDisp16,
        (false, Some((idx, is_a, scale, long))) => Ea68000::DispIdx {
            an,
            idx,
            is_a,
            scale,
            long,
        },
        (false, None) => Ea68000::Disp16(an),
    };
    Some((ea, rest, expr))
}

/// `expr.w` / `expr.l`
fn parse_abs_suffix(input: TSpan) -> Option<(Ea68000, TSpan, Node)> {
    let (rest, (_, expr)) = ms(parse_expr)(input).ok()?;
    let (size, rest) = parse_size_suffix(rest)?;
    let ea = match size {
        Size::W => Ea68000::AbsW,
        _ => Ea68000::AbsL,
    };
    Some((ea, rest, expr))
}

/// `(expr).w` / `(expr).l`
fn parse_paren_abs(input: TSpan) -> Option<(Ea68000, TSpan, Node)> {
    let (rest, _) = TokenKind::OpenBracket.parse(input).ok()?;
    let (rest, (_, expr)) = ms(parse_expr)(rest).ok()?;
    let (rest, _) = TokenKind::CloseBracket.parse(rest).ok()?;
    let (size, rest) = parse_size_suffix(rest)?;
    let ea = match size {
        Size::W => Ea68000::AbsW,
        _ => Ea68000::AbsL,
    };
    Some((ea, rest, expr))
}

/// `#expr`
fn parse_imm(input: TSpan) -> Option<(Ea68000, TSpan, Node)> {
    let (rest, (_, expr)) = ms(preceded(TokenKind::Hash, parse_expr))(input).ok()?;
    Some((Ea68000::Imm, rest, expr))
}

// ---------------------------------------------------------------------------
// movem register lists: `D0-D7/A0-A6`, `D0/D2`, `A0`

fn parse_reglist(input: TSpan) -> Option<(MovemList, TSpan)> {
    let mut list: MovemList = Vec::new();
    let mut cur = input;
    loop {
        let (n, is_a, rest) = if let Some((n, rest)) = parse_dn(cur) {
            (n, false, rest)
        } else if let Some((n, rest)) = parse_an(cur) {
            (n, true, rest)
        } else {
            return None;
        };
        let (last, rest) = if rest.first().is_some_and(|t| t.kind == TokenKind::Minus) {
            let (rest, _) = TokenKind::Minus.parse(rest).ok()?;
            let (n2, rest) = parse_dn(rest).or_else(|| parse_an(rest))?;
            (n2, rest)
        } else {
            (n, rest)
        };
        list.push((if is_a { 1 } else { 0 }, n, last));
        match rest.first().map(|t| t.kind) {
            Some(TokenKind::Slash) => {
                let (rest, _) = TokenKind::Slash.parse(rest).ok()?;
                cur = rest;
            }
            _ => return Some((list, rest)),
        }
    }
}

// ---------------------------------------------------------------------------
// one operand

fn parse_operand(input: TSpan) -> Option<(Operand68000, TSpan, Option<Node>)> {
    if let Some((ea, rest, expr)) = parse_imm(input) {
        return Some((Operand68000::Ea(ea), rest, Some(expr)));
    }
    if let Some((ea, rest)) = parse_paren_reg(input) {
        return Some((Operand68000::Ea(ea), rest, None));
    }
    if let Some((ea, rest, expr)) = parse_paren_abs(input) {
        return Some((Operand68000::Ea(ea), rest, Some(expr)));
    }
    if let Some((ea, rest, expr)) = parse_disp_paren(input) {
        return Some((Operand68000::Ea(ea), rest, Some(expr)));
    }
    if let Some((ea, rest, expr)) = parse_abs_suffix(input) {
        return Some((Operand68000::Ea(ea), rest, Some(expr)));
    }
    if let Some((ea, rest)) = parse_reg(input) {
        return Some((Operand68000::Ea(ea), rest, None));
    }
    // special registers: sr / ccr (MadMac keywords)
    if input
        .first()
        .is_some_and(|t| t.kind == TokenKind::Identifier)
    {
        let (rest, (_, txt)) = ms(get_label_string)(input).ok()?;
        let l = txt.to_ascii_lowercase();
        let ea = match l.as_str() {
            "sr" => Some(Ea68000::Sr),
            "ccr" => Some(Ea68000::Ccr),
            _ => None,
        };
        if let Some(ea) = ea {
            return Some((Operand68000::Ea(ea), rest, None));
        }
    }
    if let Ok((rest, (_, expr))) = ms(parse_expr)(input) {
        return Some((Operand68000::Ea(Ea68000::Abs), rest, Some(expr)));
    }
    None
}

// ---------------------------------------------------------------------------
// row resolution

/// The EA mode/reg encoding of a parsed EA.
pub fn ea_mode_reg(ea: &Ea68000) -> (u8, u8) {
    match *ea {
        Ea68000::Dn(n) => (0, n),
        Ea68000::An(n) => (1, n),
        Ea68000::AnInd(n) => (2, n),
        Ea68000::AnPostInc(n) => (3, n),
        Ea68000::AnPreDec(n) => (4, n),
        Ea68000::Disp16(n) => (5, n),
        Ea68000::DispIdx { an, .. } => (6, an),
        Ea68000::AbsW | Ea68000::Abs => (7, 0),
        Ea68000::AbsL => (7, 1),
        Ea68000::PcDisp16 => (7, 2),
        Ea68000::PcDispIdx { .. } => (7, 3),
        Ea68000::Imm => (7, 4),
        // sr/ccr are never encoded as EAs; (7,7) is not a valid EA and
        // matches no ea_mask bit.
        Ea68000::Sr | Ea68000::Ccr => (7, 7),
    }
}

fn as_ea(op: &Option<Operand68000>) -> Option<&Ea68000> {
    match op {
        Some(Operand68000::Ea(ea)) => Some(ea),
        _ => None,
    }
}

fn is_dn(ea: &Ea68000) -> bool {
    matches!(ea, Ea68000::Dn(_))
}
fn is_an(ea: &Ea68000) -> bool {
    matches!(ea, Ea68000::An(_))
}
fn is_predec(ea: &Ea68000) -> bool {
    matches!(ea, Ea68000::AnPreDec(_))
}
fn is_postinc(ea: &Ea68000) -> bool {
    matches!(ea, Ea68000::AnPostInc(_))
}
fn is_imm(ea: &Ea68000) -> bool {
    matches!(ea, Ea68000::Imm)
}
fn is_sr(ea: &Ea68000) -> bool {
    matches!(ea, Ea68000::Sr)
}
fn is_ccr(ea: &Ea68000) -> bool {
    matches!(ea, Ea68000::Ccr)
}
fn is_disp16(ea: &Ea68000) -> bool {
    matches!(ea, Ea68000::Disp16(_))
}
fn is_reg(ea: &Ea68000) -> bool {
    is_dn(ea) || is_an(ea)
}

/// Is this row's register operand the special SR/CCR register? The rows
/// are `move SR,<ea>` (src = M_AM_SR) and `move <ea>,SR` (dst = M_AM_SR);
/// the generic reg_ea/ea_reg forms must not accept a Dn here.
fn special_reg(ea: &Ea68000, slot: &str) -> bool {
    match slot {
        "M_AM_SR" => is_sr(ea),
        "M_AM_CCR" => is_ccr(ea),
        _ => false,
    }
}

/// Does this form match the parsed operand shapes? The ea_mask of the
/// entry's EA operand is checked too.
fn form_matches(
    form: Form,
    insn: &emu68000::isa::Insn,
    op1: &Option<Operand68000>,
    op2: &Option<Operand68000>,
) -> bool {
    use Form::*;
    let e1 = as_ea(op1);
    let e2 = as_ea(op2);

    let ea_ok = |ea: &Ea68000, mask: u16| {
        if mask == 0 {
            return true;
        }
        let (mode, reg) = ea_mode_reg(ea);
        emu68000::isa::mode_bit(mode, reg) & mask != 0
    };

    match form {
        RegEa => {
            // move SR,<ea> / move CCR,<ea> use the same reg_ea form
            if insn.src == "M_AM_SR" || insn.src == "M_AM_CCR" {
                matches!(e1, Some(e) if special_reg(e, insn.src))
                    && matches!(e2, Some(e) if ea_ok(e, insn.ea_mask))
            } else {
                matches!(e1, Some(e) if is_dn(e)) && matches!(e2, Some(e) if ea_ok(e, insn.ea_mask))
            }
        }
        EaReg => {
            // move <ea>,SR / move <ea>,CCR
            if insn.dst == "M_AM_SR" || insn.dst == "M_AM_CCR" {
                matches!(e2, Some(e) if special_reg(e, insn.dst))
                    && matches!(e1, Some(e) if ea_ok(e, insn.ea_mask))
            } else {
                matches!(e2, Some(e) if is_dn(e)) && matches!(e1, Some(e) if ea_ok(e, insn.ea_mask))
            }
        }
        EaAddr => {
            matches!(e2, Some(e) if is_an(e)) && matches!(e1, Some(e) if ea_ok(e, insn.ea_mask))
        }
        RegReg => matches!(e1, Some(e) if is_reg(e)) && matches!(e2, Some(e) if is_reg(e)),
        Exg => matches!(e1, Some(e) if is_reg(e)) && matches!(e2, Some(e) if is_reg(e)),
        PredecPredec => {
            matches!(e1, Some(e) if is_predec(e)) && matches!(e2, Some(e) if is_predec(e))
        }
        PostincPostinc => {
            matches!(e1, Some(e) if is_postinc(e)) && matches!(e2, Some(e) if is_postinc(e))
        }
        ImmEa | BitImmEa => {
            matches!(e1, Some(e) if is_imm(e)) && matches!(e2, Some(e) if ea_ok(e, insn.ea_mask))
        }
        ImmOnly => matches!(e1, Some(e) if is_imm(e)) && op2.is_none(),
        Move => {
            matches!(e1, Some(e) if ea_ok(e, insn.ea_mask))
                && matches!(e2, Some(e) if ea_ok(e, insn.ea_mask))
        }
        Moveq => matches!(e1, Some(e) if is_imm(e)) && matches!(e2, Some(e) if is_dn(e)),
        Addq => {
            matches!(e1, Some(e) if is_imm(e)) && matches!(e2, Some(e) if ea_ok(e, insn.ea_mask))
        }
        ShiftImm => matches!(e1, Some(e) if is_imm(e)) && matches!(e2, Some(e) if is_dn(e)),
        ShiftReg => matches!(e1, Some(e) if is_dn(e)) && matches!(e2, Some(e) if is_dn(e)),
        ShiftEa => matches!(e1, Some(e) if ea_ok(e, insn.ea_mask)) && op2.is_none(),
        Bcc8 | Bcc16 => matches!(e1, Some(_)) && op2.is_none(),
        Dbcc => matches!(e1, Some(e) if is_dn(e)) && matches!(e2, Some(_)),
        Scc => matches!(e1, Some(e) if ea_ok(e, insn.ea_mask)) && op2.is_none(),
        MovemRe => {
            matches!(op1, Some(Operand68000::Movem(_)))
                && matches!(e2, Some(e) if !is_predec(e) && ea_ok(e, insn.ea_mask))
        }
        MovemPd => {
            matches!(op1, Some(Operand68000::Movem(_))) && matches!(e2, Some(e) if is_predec(e))
        }
        MovemEr => {
            matches!(e1, Some(e) if ea_ok(e, insn.ea_mask))
                && matches!(op2, Some(Operand68000::Movem(_)))
        }
        Link => matches!(e1, Some(e) if is_an(e)) && matches!(e2, Some(e) if is_imm(e)),
        Movep => {
            (matches!(e1, Some(e) if is_disp16(e)) && matches!(e2, Some(e) if is_dn(e)))
                || (matches!(e1, Some(e) if is_dn(e)) && matches!(e2, Some(e) if is_disp16(e)))
        }
        Trap => matches!(e1, Some(e) if is_imm(e)) && op2.is_none(),
        Ea => matches!(e1, Some(e) if ea_ok(e, insn.ea_mask)) && op2.is_none(),
        Dreg => matches!(e1, Some(e) if is_dn(e)) && op2.is_none(),
        Areg => matches!(e1, Some(e) if is_an(e)) && op2.is_none(),
        Imm16 => matches!(e1, Some(e) if is_imm(e)) && op2.is_none(),
        None => op1.is_none() && op2.is_none(),
    }
}

// ---------------------------------------------------------------------------
// entry

/// Parse one operand pair; returns the matched row, the parsed operands,
/// the expression children, and the rest.
fn parse_opcode_operands<'a>(
    input: TSpan<'a>,
    sp: TSpan<'a>,
    rows: Vec<usize>,
) -> PResult<'a, (usize, Instr68000, Vec<Node>)> {
    let is_movem = rows.iter().any(|&i| {
        matches!(
            table().row(i).form,
            Form::MovemRe | Form::MovemEr | Form::MovemPd
        )
    });

    let (rest, (op1, expr1)) = if is_movem {
        if let Some((list, rest)) = parse_reglist(input) {
            (rest, (Some(Operand68000::Movem(list)), None))
        } else {
            match parse_operand(input) {
                Some((op, rest, expr)) => (rest, (Some(op), expr)),
                None => return err_fatal(sp, CpuAssemblyErrorKind::OperandsDontMatch),
            }
        }
    } else {
        match parse_operand(input) {
            Some((op, rest, expr)) => (rest, (Some(op), expr)),
            None => return err_fatal(sp, CpuAssemblyErrorKind::OperandsDontMatch),
        }
    };

    let (rest, (op2, expr2)) = match rest.first().map(|t| t.kind) {
        Some(TokenKind::Comma) => {
            let (rest, _) = TokenKind::Comma.parse(rest).ok().unwrap();
            if is_movem && matches!(op1, Some(Operand68000::Ea(_))) {
                match parse_reglist(rest) {
                    Some((list, rest)) => (rest, (Some(Operand68000::Movem(list)), None)),
                    None => return err_fatal(sp, CpuAssemblyErrorKind::OperandsDontMatch),
                }
            } else {
                match parse_operand(rest) {
                    Some((op, rest, expr)) => (rest, (Some(op), expr)),
                    None => return err_fatal(sp, CpuAssemblyErrorKind::OperandsDontMatch),
                }
            }
        }
        _ => (rest, (None, None)),
    };

    let ins = Instr68000 {
        op1,
        op2,
        short: false,
    };
    let row = rows
        .iter()
        .copied()
        .find(|&i| form_matches(table().row(i).form, table().row(i), &ins.op1, &ins.op2));

    let row = match row {
        Some(r) => r,
        None => return err_fatal(sp, CpuAssemblyErrorKind::OperandsDontMatch),
    };

    let mut children = Vec::new();
    if let Some(e) = expr1 {
        children.push(e);
    }
    if let Some(e) = expr2 {
        children.push(e);
    }

    Ok((rest, (row, ins, children)))
}

pub fn parse_opcode(input: TSpan) -> PResult<Node> {
    // mnemonic (an identifier or classified opcode) plus an optional
    // trailing `.b`/`.w`/`.l` suffix token.
    let (rest, sp, mn, suffix) = {
        use TokenKind::{CpuOpcode, Identifier, Label};
        let (rest, (msp, matched)) = ms(alt((CpuOpcode(Cpu), Identifier, Label)))(input)?;
        let text = get_text(matched).to_ascii_lowercase();
        let mut rest = rest;
        let mut suffix = None;
        if let Some((s, r)) = parse_size_suffix(rest) {
            suffix = Some(s);
            rest = r;
        }
        let (mn, embedded) = match text.rsplit_once('.') {
            Some((m, s)) if suffix_size(&format!(".{s}")).is_some() => {
                (m.to_string(), suffix_size(&format!(".{s}")))
            }
            _ => (text, None),
        };
        if suffix.is_none() {
            suffix = embedded;
        }
        (rest, msp, mn, suffix)
    };

    // dc.b / dc.w / dc.l data directives and ds.b / ds.w / ds.l space
    // reservations (MadMac spellings).
    if mn == "dc" || mn == "ds" {
        let size = suffix.ok_or_else(|| fatal(sp, CpuAssemblyErrorKind::MissingOperand))?;
        let item = match (mn.as_str(), size) {
            ("dc", Size::B) => AstNodeKind::EmitBytes(0),
            ("dc", Size::W) => AstNodeKind::EmitWords(0),
            ("dc", Size::L) => AstNodeKind::EmitLongs(0),
            ("ds", Size::B) => AstNodeKind::ReserveBytes,
            ("ds", Size::W) => AstNodeKind::ReserveWords,
            ("ds", Size::L) => AstNodeKind::ReserveLongs,
            _ => return err_fatal(sp, CpuAssemblyErrorKind::MissingOperand),
        };
        let (rest, (esp, exprs)) = if mn == "dc" {
            ms(crate::frontend::GazmParser::parse_expr_list)(rest)?
        } else {
            let (rest, (esp, e)) = ms(crate::frontend::parse_expr)(rest)?;
            (rest, (esp, vec![e]))
        };
        let node = match item {
            AstNodeKind::EmitBytes(_) => {
                let len = exprs.len();
                from_item_children_tspan(AstNodeKind::EmitBytes(len), &exprs, esp)
            }
            AstNodeKind::EmitWords(_) => {
                let len = exprs.len();
                from_item_children_tspan(AstNodeKind::EmitWords(len), &exprs, esp)
            }
            AstNodeKind::EmitLongs(_) => {
                let len = exprs.len();
                from_item_children_tspan(AstNodeKind::EmitLongs(len), &exprs, esp)
            }
            _ => from_item_children_tspan(item, &exprs, esp),
        };
        return Ok((rest, node));
    }

    let mut rows = table()
        .lookup(&mn, suffix)
        .ok_or_else(|| fatal(sp, CpuAssemblyErrorKind::UnknownOpcode("68000")))?;

    // A `.b`/`.s` suffix on a branch mnemonic forces the 8-bit form
    // (MadMac m_br: SIZB/SIZS = short; the bare form auto-extends).
    let mut short = false;
    if suffix == Some(Size::B) {
        if let Some(bare) = table().lookup(&mn, None) {
            let bcc8: Vec<usize> = bare
                .iter()
                .copied()
                .filter(|&i| table().row(i).form == Form::Bcc8)
                .collect();
            if !bcc8.is_empty() {
                short = true;
                rows = bcc8;
            }
        }
    }

    // no operands: inherent forms (nop, rts, dc.w special-cased below)
    if rest.is_empty() {
        let row = rows
            .iter()
            .copied()
            .find(|&i| table().row(i).form == Form::None);
        if let Some(row) = row {
            if table().row(row).mnemonic == "dc.w" {
                return err_fatal(sp, CpuAssemblyErrorKind::OperandsDontMatch);
            }
            let item = OpCode(
                row,
                Instr68000 {
                    op1: None,
                    op2: None,
                    short: false,
                },
            );
            return Ok((rest, from_item_tspan(item, sp)));
        }
        return err_fatal(sp, CpuAssemblyErrorKind::OperandsDontMatch);
    }

    let (rest, (row, mut ins, children)) = match parse_opcode_operands(rest, sp, rows.clone()) {
        Ok(x) => x,
        // MadMac: `sub #imm,<ea>` assembles as `subi` and `cmp a1,a0`
        // as `cmpa` (the base mnemonic lacks those forms). Retry with
        // each alias until one matches.
        Err(_) => {
            let aliases: [Option<Vec<usize>>; 2] = [
                table().alias_rows(&mn, suffix),
                table().an_alias_rows(&mn, suffix),
            ];
            let mut matched = None;
            for candidate in aliases.into_iter().flatten() {
                if let Ok((rest2, (row2, ins2, children2))) =
                    parse_opcode_operands(rest, sp, candidate)
                {
                    let mut ins = ins2;
                    ins.short = short;
                    let node = from_item_children_tspan(OpCode(row2, ins), &children2, sp);
                    matched = Some((rest2, node));
                    break;
                }
            }
            match matched {
                Some((rest2, node)) => return Ok((rest2, node)),
                None => return err_fatal(sp, CpuAssemblyErrorKind::OperandsDontMatch),
            }
        }
    };
    ins.short = short;
    let item = OpCode(row, ins);
    let node = from_item_children_tspan(item, &children, sp);
    Ok((rest, node))
}

pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, opcode) = parse_opcode(input)?;
    Ok((rest, vec![opcode]))
}

/// Build a symbol expression node for a bare identifier (used by the
/// sizer when widening a bare absolute).
pub fn symbol_node(text: &str, sp: TSpan) -> Node {
    from_item_tspan(
        AstNodeKind::Label(LabelDefinition::Text(text.to_string())),
        sp,
    )
}

#[allow(unused_imports)]
use CpuSpecific::Cpu68000 as CpuSpec;
