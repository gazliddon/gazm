//! The Jaguar RISC (GPU/DSP) frontend: instruction parsing.
//!
//! The JRISC instruction set is a fixed-width 16-bit word with two 5-bit
//! register fields, so operand *forms* are decided by the mnemonic's type
//! (see [`JriscType`]) and the encodable details (register numbers, quick
//! immediates, index-base choice) are resolved at assembly time from the
//! operand expressions — register names can be `r0..r31` literals or
//! `REGEQU`/`EQU` symbols evaluating to 0..31, exactly like rmac.

use crate::frontend::{
    err_fatal, fatal, from_item_child_tspan, from_item_children_tspan, from_item_tspan,
    get_label_string, get_text, parse_expr, AstNodeKind, CpuAssemblyErrorKind, Node, PResult,
    TSpan, TokenKind,
};

use crate::cpujrisc::isa::{cond, find, JriscOp, JriscType};
use crate::cpukind::CpuKind::CpuJagrisc as Cpu;

use unraveler::{alt, match_span as ms, preceded, Collection, Parser};

/// A register operand position: a literal `rN` or an expression child
/// (REGEQU/EQU name) that must evaluate to 0..31.
#[derive(Debug, Clone, PartialEq)]
pub enum JriscReg {
    Lit(u8),
    Expr(usize),
}

/// A JRISC operand. Expression-bearing operands consume the instruction
/// node's children in order; the index records the child slot.
#[derive(Debug, Clone, PartialEq)]
pub enum JriscOperand {
    /// A literal `r0..r31` register — no expression child.
    RegLit(u8),
    /// A register given by an expression (REGEQU/EQU name that must
    /// evaluate to 0..31) — one child.
    RegExpr(usize),
    /// `PC` (move's source) — no child.
    Pc,
    /// `#expr` immediate — one child.
    Imm(usize),
    /// `(reg + rN)` — literal index register — no children.
    ParenIndex { base: JriscReg, index: u8 },
    /// `(reg + expr)` — base register plus constant offset expression.
    ParenSum { base: JriscReg, offset: usize },
    /// A condition code resolved at parse time — no child.
    Cond(u8),
    /// The `jr` target expression — one child.
    Target(usize),
}

/// A parsed JRISC instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct JriscInsnAst {
    pub op: &'static JriscOp,
    pub op1: Option<JriscOperand>,
    pub op2: Option<JriscOperand>,
}

/// The JRISC node kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKindJrisc {
    OpCode(JriscInsnAst),
}

impl From<NodeKindJrisc> for crate::frontend::CpuSpecific {
    fn from(value: NodeKindJrisc) -> Self {
        crate::frontend::CpuSpecific::CpuJagrisc(value)
    }
}

impl From<NodeKindJrisc> for crate::frontend::AstNodeKind {
    fn from(value: NodeKindJrisc) -> Self {
        crate::frontend::AstNodeKind::TargetSpecific(crate::frontend::CpuSpecific::CpuJagrisc(
            value,
        ))
    }
}

/// Classify an identifier for the JRISC backend: the mnemonic vocabulary
/// becomes `CpuOpcode`, everything else is a label.
pub fn lex_identifier(text: &str) -> TokenKind {
    use TokenKind::*;
    let text = crate::frontend::ascii_lowercase(text);
    if find(text.as_ref()).is_some() {
        CpuOpcode(Cpu)
    } else {
        Label
    }
}

/// A literal `r0..r31` register token.
fn parse_reg_lit(input: TSpan) -> Option<(u8, TSpan)> {
    let (rest, (sp, _)) = ms(get_label_string)(input).ok()?;
    let t = get_text(sp).to_ascii_uppercase();
    if t.starts_with('R') {
        if let Ok(n) = t[1..].parse::<u8>() {
            if n < 32 {
                return Some((n, rest));
            }
        }
    }
    None
}

/// A register operand: `rN` literal, or an expression (REGEQU/EQU symbol)
/// that must evaluate to a register number. Pushes the expression child
/// when needed.
fn reg_operand<'a>(
    input: TSpan<'a>,
    children: &mut Vec<Node>,
) -> Option<(JriscOperand, TSpan<'a>)> {
    if let Some((n, rest)) = parse_reg_lit(input) {
        return Some((JriscOperand::RegLit(n), rest));
    }
    let (rest, expr) = parse_expr(input).ok()?;
    children.push(expr);
    Some((JriscOperand::RegExpr(children.len() - 1), rest))
}

/// A `#expr` immediate operand (with the `#` already consumed).
fn imm_after_hash<'a>(
    children: &mut Vec<Node>,
    input: TSpan<'a>,
) -> Option<(JriscOperand, TSpan<'a>)> {
    let (rest, expr) = parse_expr(input).ok()?;
    children.push(expr);
    Some((JriscOperand::Imm(children.len() - 1), rest))
}

/// A parenthesised operand: `(reg)`, `(reg + offset)` or `(reg + rN)`.
/// The plain `(reg)` form is returned as [`JriscOperand::RegLit`] /
/// [`JriscOperand::RegExpr`] (the enclosing mnemonic type tells the
/// assembler it was parenthesised).
fn paren_operand<'a>(
    input: TSpan<'a>,
    children: &mut Vec<Node>,
) -> Option<(JriscOperand, TSpan<'a>)> {
    let (rest, _) = TokenKind::OpenBracket.parse(input).ok()?;

    // The base must be a register name (rN literal or a single REGEQU/
    // EQU symbol); the offset after '+' is a full expression.
    let (base, after_base): (JriscReg, TSpan) = if let Some((n, rest)) = parse_reg_lit(rest) {
        (JriscReg::Lit(n), rest)
    } else if rest
        .first()
        .is_some_and(|t| matches!(t.kind, TokenKind::Identifier | TokenKind::Label))
    {
        let (rest, (sp, _)) = ms(get_label_string)(rest).ok()?;
        let node = from_item_tspan(
            AstNodeKind::Label(crate::frontend::LabelDefinition::Text(get_text(sp))),
            sp,
        );
        children.push(node);
        (JriscReg::Expr(children.len() - 1), rest)
    } else {
        return None;
    };

    let (operand, after_plus) = if after_base
        .first()
        .is_some_and(|t| t.kind == TokenKind::Plus)
    {
        let (rest, _) = TokenKind::Plus.parse(after_base).ok()?;
        if let Some((n, rest)) = parse_reg_lit(rest) {
            (
                JriscOperand::ParenIndex {
                    base: base.clone(),
                    index: n,
                },
                rest,
            )
        } else {
            let (rest, expr) = parse_expr(rest).ok()?;
            children.push(expr);
            (
                JriscOperand::ParenSum {
                    base: base.clone(),
                    offset: children.len() - 1,
                },
                rest,
            )
        }
    } else {
        (
            match base {
                JriscReg::Lit(n) => JriscOperand::RegLit(n),
                JriscReg::Expr(i) => JriscOperand::RegExpr(i),
            },
            after_base,
        )
    };

    let (rest, _) = TokenKind::CloseBracket.parse(after_plus).ok()?;
    Some((operand, rest))
}

/// A condition-code name in the first operand position.
fn parse_cond(input: TSpan) -> Option<(u8, TSpan)> {
    let (rest, (sp, _)) = ms(get_label_string)(input).ok()?;
    let name = get_text(sp);
    cond(&name).map(|n| (n, rest))
}

/// Parse one JRISC instruction. Returns the node and the rest of the input.
pub fn parse_opcode(input: TSpan) -> PResult<Node> {
    use TokenKind::{Comma, CpuOpcode, Identifier};

    let (rest, (msp, matched)) = ms(alt((CpuOpcode(Cpu), Identifier)))(input)?;
    let name = get_text(matched).to_ascii_lowercase();
    let op = find(&name).ok_or_else(|| fatal(msp, CpuAssemblyErrorKind::UnknownOpcode("JRISC")))?;

    let mut children: Vec<Node> = Vec::new();
    let mut rest = rest;
    let mut op1: Option<JriscOperand> = None;
    let mut op2: Option<JriscOperand> = None;

    let missing = || fatal(msp, CpuAssemblyErrorKind::MissingOperand);

    match op.typ {
        JriscType::None => {}

        JriscType::One => {
            let (opd, r) = reg_operand(rest, &mut children).ok_or_else(missing)?;
            op2 = Some(opd);
            rest = r;
        }

        JriscType::Two => {
            let (opd, r) = reg_operand(rest, &mut children).ok_or_else(missing)?;
            op1 = Some(opd);
            rest = r;
            let (r, _) = Comma.parse(rest)?;
            rest = r;
            let (opd, r) = reg_operand(rest, &mut children).ok_or_else(missing)?;
            op2 = Some(opd);
            rest = r;
        }

        JriscType::Num15 | JriscType::Num31 | JriscType::Num32 => {
            let (r, _) = TokenKind::Hash.parse(rest).map_err(|_| missing())?;
            rest = r;
            let (opd, r) = imm_after_hash(&mut children, rest).ok_or_else(missing)?;
            op1 = Some(opd);
            rest = r;
            let (r, _) = Comma.parse(rest)?;
            rest = r;
            let (opd, r) = reg_operand(rest, &mut children).ok_or_else(missing)?;
            op2 = Some(opd);
            rest = r;
        }

        JriscType::Movei => {
            let (r, _) = TokenKind::Hash.parse(rest).map_err(|_| missing())?;
            rest = r;
            let (opd, r) = imm_after_hash(&mut children, rest).ok_or_else(missing)?;
            op1 = Some(opd);
            rest = r;
            let (r, _) = Comma.parse(rest)?;
            rest = r;
            let (opd, r) = reg_operand(rest, &mut children).ok_or_else(missing)?;
            op2 = Some(opd);
            rest = r;
        }

        JriscType::Move => {
            // `PC` or a register as the source.
            if rest
                .first()
                .is_some_and(|t| t.kind == TokenKind::Identifier)
                && {
                    let (_, (_, txt)) = ms(get_label_string)(rest).ok().unwrap();
                    txt.eq_ignore_ascii_case("pc")
                }
            {
                let (r, _) = ms(get_label_string)(rest)?;
                op1 = Some(JriscOperand::Pc);
                rest = r;
            } else {
                let (opd, r) = reg_operand(rest, &mut children).ok_or_else(missing)?;
                op1 = Some(opd);
                rest = r;
            }
            let (r, _) = Comma.parse(rest)?;
            rest = r;
            let (opd, r) = reg_operand(rest, &mut children).ok_or_else(missing)?;
            op2 = Some(opd);
            rest = r;
        }

        JriscType::Load | JriscType::Loadn => {
            let (opd, r) = paren_operand(rest, &mut children).ok_or_else(missing)?;
            op1 = Some(opd);
            rest = r;
            let (r, _) = Comma.parse(rest)?;
            rest = r;
            let (opd, r) = reg_operand(rest, &mut children).ok_or_else(missing)?;
            op2 = Some(opd);
            rest = r;
        }

        JriscType::Store | JriscType::Storen => {
            let (opd, r) = reg_operand(rest, &mut children).ok_or_else(missing)?;
            op1 = Some(opd);
            rest = r;
            let (r, _) = Comma.parse(rest)?;
            rest = r;
            let (opd, r) = paren_operand(rest, &mut children).ok_or_else(missing)?;
            op2 = Some(opd);
            rest = r;
        }

        JriscType::Jr | JriscType::Jump => {
            // Optional condition code as the first operand. A leading '('
            // means "jump always" (cc = 0).
            let has_cond = rest
                .first()
                .is_some_and(|t| t.kind == TokenKind::Identifier)
                && parse_cond(rest).is_some();
            if has_cond {
                let (n, r) = parse_cond(rest).unwrap();
                op1 = Some(JriscOperand::Cond(n));
                rest = r;
                let (r, _) = Comma.parse(rest)?;
                rest = r;
            }
            if op.typ == JriscType::Jr {
                let (r, expr) = parse_expr(rest)?;
                children.push(expr);
                op2 = Some(JriscOperand::Target(children.len() - 1));
                rest = r;
            } else {
                let (opd, r) = paren_operand(rest, &mut children).ok_or_else(missing)?;
                op2 = Some(opd);
                rest = r;
            }
        }
    }

    let ins = JriscInsnAst { op, op1, op2 };
    let node = from_item_children_tspan(NodeKindJrisc::OpCode(ins), &children, msp);
    Ok((rest, node))
}

pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, opcode) = parse_opcode(input)?;
    Ok((rest, vec![opcode]))
}

/// `name REGEQU rN` — equate a name to a JRISC register (rmac). Parsed
/// like `equ` but the value is the register number, so the register field
/// evaluation in the assembler can resolve the name.
pub fn parse_reg_equate(input: TSpan) -> PResult<Node> {
    use crate::frontend::{keyword, GazmParser, ParsedFrom};
    use unraveler::opt;

    let (rest, label_node) = GazmParser::parse_label(input)?;
    let def = match label_node.item {
        AstNodeKind::Label(d) => d,
        _ => return err_fatal(input, CpuAssemblyErrorKind::MissingOperand),
    };

    let (rest, _) = opt(TokenKind::Colon).parse(rest)?;
    let (rest, _) = keyword("regequ")(rest)?;
    let (rest, (tsp, _)) = ms(get_label_string)(rest)?;

    let t = get_text(tsp).to_ascii_uppercase();
    let n = t
        .strip_prefix('R')
        .and_then(|d| d.parse::<u8>().ok())
        .filter(|n| *n < 32)
        .ok_or_else(|| {
            crate::frontend::FrontEndError::error(
                tsp,
                crate::frontend::FrontEndErrorKind::Unexpected,
            )
        })?;

    let value = from_item_tspan(AstNodeKind::Num(n as i64, ParsedFrom::Decimal), tsp);
    let node = from_item_child_tspan(AstNodeKind::Assignment(def), value, input);
    Ok((rest, node))
}

/// Build a symbol expression node for a bare identifier (used by the
/// assembler when resolving an absolute).
pub fn symbol_node(text: &str, sp: TSpan) -> Node {
    from_item_tspan(
        AstNodeKind::Label(crate::frontend::LabelDefinition::Text(text.to_string())),
        sp,
    )
}
