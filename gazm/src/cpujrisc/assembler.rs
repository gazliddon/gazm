//! The Jaguar RISC (GPU/DSP) assembler: sizing and encoding.
//!
//! Every JRISC instruction is one 16-bit word (`op<<10 | r1<<5 | r2`);
//! `movei` adds a second word pair holding the 32-bit immediate, stored
//! low word first (rmac's `WORDSWAP32` long). `jr` encodes a word
//! displacement relative to the instruction address; everything else
//! resolves from operand expressions at assembly time.

use crate::assembler::{Assembler, Sizer};
use crate::cpujrisc::frontend::{JriscInsnAst, JriscOperand, JriscReg, NodeKindJrisc};
use crate::cpujrisc::isa::{JriscType, INDEX_REGS};
use crate::error::GResult;
use crate::frontend::CpuSpecific;
use crate::semantic::{AstNodeId, AstNodeRef};

use NodeKindJrisc::OpCode;

/// Size a JRISC node: 2 bytes per instruction word; `movei` is 6
/// (opcode word + 32-bit immediate).
pub fn size_node_internal(
    sizer: &mut Sizer,
    _asm: &mut Assembler,
    _id: AstNodeId,
    node_kind: NodeKindJrisc,
) -> GResult<()> {
    let OpCode(ins) = node_kind;
    let size = if ins.op.typ == JriscType::Movei { 6 } else { 2 };
    sizer.advance_pc(size);
    Ok(())
}

/// Evaluate the next expression child of the instruction node.
fn eval_child(asm: &mut Assembler, node: AstNodeRef, scope: u64, child: &mut usize) -> Option<i64> {
    let c = node.children().nth(*child)?;
    *child += 1;
    asm.eval_node(c, scope).ok()
}

/// Resolve a register operand to a number; `what` describes the operand
/// for the error message (rmac: "missing register R0...R31").
fn eval_reg(
    asm: &mut Assembler,
    node: AstNodeRef,
    scope: u64,
    child: &mut usize,
    op: &JriscOperand,
    what: &str,
) -> GResult<u8> {
    let v = match op {
        JriscOperand::RegLit(n) => *n as i64,
        JriscOperand::RegExpr(_) => eval_child(asm, node, scope, child)
            .ok_or_else(|| asm.make_user_error("missing register R0...R31", node, true))?,
        _ => return Err(asm.make_user_error("register expected", node, true).into()),
    };
    if !(0..=31).contains(&v) {
        return Err(asm
            .make_user_error(
                format!("{what}: missing register R0...R31 ({v})"),
                node,
                true,
            )
            .into());
    }
    Ok(v as u8)
}

/// Compile a JRISC node (registry entry: `crate::cpujrisc::assembler::compile_node`).
pub fn compile_node(
    asm: &mut Assembler,
    node: AstNodeRef,
    node_kind: NodeKindJrisc,
    current_scope_id: u64,
) -> GResult<()> {
    let OpCode(ins) = node_kind;
    let pc = asm.get_binary().get_write_address();
    let mut child = 0usize;
    if ins.op.typ == JriscType::Movei {
        // Children are in source order: the immediate expression comes
        // first, then the destination register.
        let imm = eval_child(asm, node, current_scope_id, &mut child)
            .ok_or_else(|| asm.make_user_error("missing movei immediate", node, true))?;
        let word = encode(asm, node, &ins, pc, current_scope_id, &mut child)?;
        asm.write_word(word, node)?;
        // Low word first, then the high word (rmac WORDSWAP32 long).
        asm.write_word((imm & 0xffff) as u16, node)?;
        asm.write_word(((imm >> 16) & 0xffff) as u16, node)?;
    } else {
        let word = encode(asm, node, &ins, pc, current_scope_id, &mut child)?;
        asm.write_word(word, node)?;
    }
    Ok(())
}

fn encode(
    asm: &mut Assembler,
    node: AstNodeRef,
    ins: &JriscInsnAst,
    pc: usize,
    scope: u64,
    child: &mut usize,
) -> GResult<u16> {
    use JriscOperand::*;
    let op = ins.op;
    let base = (op.op & 0x3f) << 10;

    // Register fields from the operands (order varies by type).
    macro_rules! regs {
        ($o1:expr, $o2:expr) => {{
            let r1 = eval_reg(asm, node, scope, child, $o1, "operand 1")?;
            let r2 = eval_reg(asm, node, scope, child, $o2, "operand 2")?;
            (r1, r2)
        }};
    }

    match op.typ {
        JriscType::None => Ok(base),

        JriscType::One => {
            let rd = match ins.op2.as_ref().unwrap() {
                RegLit(n) => *n as i64,
                RegExpr(_) => eval_child(asm, node, scope, child)
                    .ok_or_else(|| asm.make_user_error("missing register", node, true))?,
                _ => return Err(asm.make_user_error("register expected", node, true).into()),
            };
            if !(0..=31).contains(&rd) {
                return Err(asm
                    .make_user_error(format!("missing register R0...R31 ({rd})"), node, true)
                    .into());
            }
            // rmac stores bit 6 of the opcode parameter in reg1 (pack).
            Ok(base | (((op.op >> 6) & 0x1f) << 5) | rd as u16)
        }

        JriscType::Two => {
            let (r1, r2) = regs!(ins.op1.as_ref().unwrap(), ins.op2.as_ref().unwrap());
            Ok(base | ((r1 as u16) << 5) | r2 as u16)
        }

        JriscType::Num15 | JriscType::Num31 | JriscType::Num32 => {
            let imm = match ins.op1.as_ref().unwrap() {
                Imm(_) => eval_child(asm, node, scope, child)
                    .ok_or_else(|| asm.make_user_error("missing immediate", node, true))?,
                _ => return Err(asm.make_user_error("immediate expected", node, true).into()),
            };
            let (lo, hi, what) = match op.typ {
                JriscType::Num15 => (-16i64, 15i64, "constant out of range (-16 to 15)"),
                JriscType::Num31 => (0i64, 31i64, "constant out of range (0 to 31)"),
                _ => (1i64, 32i64, "constant out of range (1 to 32)"),
            };
            if !(lo..=hi).contains(&imm) {
                return Err(asm
                    .make_user_error(format!("{what} ({imm})"), node, true)
                    .into());
            }
            let field = if op.sub32 {
                32 - imm
            } else if op.typ == JriscType::Num32 && imm == 32 {
                0
            } else {
                imm
            };
            let rd = match ins.op2.as_ref().unwrap() {
                RegLit(n) => *n as i64,
                RegExpr(_) => eval_child(asm, node, scope, child)
                    .ok_or_else(|| asm.make_user_error("missing register", node, true))?,
                _ => return Err(asm.make_user_error("register expected", node, true).into()),
            };
            if !(0..=31).contains(&rd) {
                return Err(asm
                    .make_user_error(format!("missing register R0...R31 ({rd})"), node, true)
                    .into());
            }
            Ok(base | (((field as u16) & 0x1f) << 5) | rd as u16)
        }

        JriscType::Movei => {
            // First word: opcode with reg2 = destination, reg1 = 0.
            let rd = match ins.op2.as_ref().unwrap() {
                RegLit(n) => *n as i64,
                RegExpr(_) => eval_child(asm, node, scope, child)
                    .ok_or_else(|| asm.make_user_error("missing register", node, true))?,
                _ => return Err(asm.make_user_error("register expected", node, true).into()),
            };
            if !(0..=31).contains(&rd) {
                return Err(asm
                    .make_user_error(format!("missing register R0...R31 ({rd})"), node, true)
                    .into());
            }
            Ok(base | rd as u16)
        }

        JriscType::Move => {
            // `move pc,rd` -> opcode 51; `move rs,rd` -> opcode 34.
            let (opcode, r1) = match ins.op1.as_ref().unwrap() {
                Pc => (51u16, 0u16),
                _ => {
                    let rs =
                        eval_reg(asm, node, scope, child, ins.op1.as_ref().unwrap(), "source")?;
                    (34u16, rs as u16)
                }
            };
            let rd = eval_reg(
                asm,
                node,
                scope,
                child,
                ins.op2.as_ref().unwrap(),
                "destination",
            )?;
            Ok((opcode << 10) | (r1 << 5) | rd as u16)
        }

        JriscType::Load | JriscType::Loadn => {
            // (Rn),Rd / (R14|R15 + n),Rd / (R14|R15 + rN),Rd.
            // loadb/loadw/loadp have their own fixed opcodes; the plain
            // `load` polymorphic form resolves the index variants.
            let plain = if op.typ == JriscType::Loadn {
                op.op
            } else {
                41
            };
            let (opcode, r1) =
                load_store_index(asm, node, scope, child, ins.op1.as_ref().unwrap(), plain)?;
            let rd = eval_reg(
                asm,
                node,
                scope,
                child,
                ins.op2.as_ref().unwrap(),
                "destination",
            )?;
            Ok((opcode << 10) | ((r1 as u16) << 5) | rd as u16)
        }

        JriscType::Store | JriscType::Storen => {
            // Rs,(Rn) — note the swapped register fields.
            let rs = eval_reg(asm, node, scope, child, ins.op1.as_ref().unwrap(), "source")?;
            let plain = if op.typ == JriscType::Storen {
                op.op
            } else {
                47
            };
            let (opcode, r1) =
                load_store_index(asm, node, scope, child, ins.op2.as_ref().unwrap(), plain)?;
            Ok((opcode << 10) | ((r1 as u16) << 5) | rs as u16)
        }

        JriscType::Jr => {
            let cc = match ins.op1.as_ref() {
                Some(Cond(n)) => *n as u16,
                _ => 0,
            };
            let target = eval_child(asm, node, scope, child)
                .ok_or_else(|| asm.make_user_error("missing jump target", node, true))?;
            let disp = (target - (pc as i64 + 2)) / 2;
            if !(-16..=15).contains(&disp) {
                return Err(asm
                    .make_user_error(
                        format!("PC relative overflow in JR (outside of -16 to 15): {disp}"),
                        node,
                        true,
                    )
                    .into());
            }
            Ok(base | (((disp as u16) & 0x1f) << 5) | cc)
        }

        JriscType::Jump => {
            let cc = match ins.op1.as_ref() {
                Some(Cond(n)) => *n as u16,
                _ => 0,
            };
            // The ROM's encoding puts the register in reg1 and the
            // condition in reg2 (`jump mi,(r0)` = $D018: reg1=0, reg2=24).
            let (opcode, rn) =
                load_store_index(asm, node, scope, child, ins.op2.as_ref().unwrap(), 52)?;
            Ok((opcode << 10) | ((rn as u16) << 5) | cc)
        }
    }
}

/// Resolve a `(reg)` / `(reg + offset)` / `(reg + rN)` operand to the
/// (opcode, reg1 field) pair. `plain_op` is the opcode for the unindexed
/// form; `store_like` swaps the meaning of the field for stores (the field
/// holds the destination base, the other operand the source).
fn load_store_index(
    asm: &mut Assembler,
    node: AstNodeRef,
    scope: u64,
    child: &mut usize,
    op: &JriscOperand,
    plain_op: u16,
) -> GResult<(u16, u8)> {
    // Indexed opcodes follow the plain form: load 41 -> 43/44 (R14/R15),
    // store 47 -> 49/50; register-indexed load 58/59, store 60/61.
    let (disp_op, reg_op) = match plain_op {
        41 => (43u16, 58u16),
        47 => (49u16, 60u16),
        _ => (0, 0),
    };
    let _ = (disp_op, reg_op);
    use JriscOperand::*;
    match op {
        // (Rn)
        RegLit(n) => Ok((plain_op, *n)),
        RegExpr(_) => {
            let v = eval_child(asm, node, scope, child)
                .ok_or_else(|| asm.make_user_error("missing register", node, true))?;
            if !(0..=31).contains(&v) {
                return Err(asm
                    .make_user_error(format!("missing register R0...R31 ({v})"), node, true)
                    .into());
            }
            Ok((plain_op, v as u8))
        }
        // (R14|R15 + n) / (R14|R15 + rN)
        ParenSum { base, .. } | ParenIndex { base, .. } => {
            let is_index = matches!(op, ParenIndex { .. });
            let base_v = match base {
                JriscReg::Lit(n) => *n as i64,
                JriscReg::Expr(_) => eval_child(asm, node, scope, child)
                    .ok_or_else(|| asm.make_user_error("missing base register", node, true))?,
            };
            if !INDEX_REGS.contains(&(base_v as u8)) {
                return Err(asm
                    .make_user_error(
                        format!(
                            "Attempted index reference with non-indexable register (r{base_v})"
                        ),
                        node,
                        true,
                    )
                    .into());
            }
            let r14 = INDEX_REGS[0] as u16;
            if is_index {
                let idx = match op {
                    ParenIndex { index, .. } => *index,
                    _ => unreachable!(),
                };
                // reg_op/reg_op+1: base + register index.
                let opcode = if base_v as u16 == r14 {
                    reg_op
                } else {
                    reg_op + 1
                };
                Ok((opcode, idx))
            } else {
                let off = eval_child(asm, node, scope, child)
                    .ok_or_else(|| asm.make_user_error("missing offset", node, true))?;
                if off == 0 {
                    // rmac: "NULL offset in LOAD/STORE ignored" — plain form.
                    return Ok((plain_op, base_v as u8));
                }
                if !(1..=32).contains(&off) {
                    return Err(asm
                        .make_user_error(
                            format!("constant in LOAD out of range (1-32): {off}"),
                            node,
                            true,
                        )
                        .into());
                }
                let field = if off == 32 { 0 } else { off as u8 };
                let opcode = if base_v as u16 == r14 {
                    disp_op
                } else {
                    disp_op + 1
                };
                Ok((opcode, field))
            }
        }
        _ => Err(asm
            .make_user_error("expected (register)", node, true)
            .into()),
    }
}

#[allow(unused_imports)]
use CpuSpecific::CpuJagrisc as CpuSpec;
