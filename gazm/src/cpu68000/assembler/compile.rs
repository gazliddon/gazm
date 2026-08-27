#![deny(unused_imports)]
use crate::assembler::Assembler;
use crate::cpu68000::{
    frontend::{ea_mode_reg, Ea68000, NodeKind68000, Operand68000},
    isa::table,
};
use crate::error::GResult;
use crate::semantic::AstNodeRef;

use emu68000::isa::{Form, Size};

use NodeKind68000::OpCode;

/// The EA field (mode<<3|reg) for the low six bits of an opcode word.
fn ea_bits(ea: &Ea68000) -> u16 {
    let (mode, reg) = ea_mode_reg(ea);
    ((mode as u16) << 3) | reg as u16
}

/// Evaluate the next expression child (operands consume their children in
/// order, mirroring the sizer).
fn eval_child(asm: &mut Assembler, node: AstNodeRef, scope: u64, child: &mut usize) -> Option<i64> {
    let c = node.children().nth(*child)?;
    *child += 1;
    asm.eval_node(c, scope).ok()
}

/// 68000 branch displacement: target - (pc + 2). Range errors honour
/// `ignore_relative_offset_errors`; MadMac also rejects a zero short
/// displacement.
fn rel_disp(
    asm: &mut Assembler,
    node: AstNodeRef,
    target: i64,
    pc: usize,
    bits: u8,
    madmac: bool,
) -> GResult<i16> {
    let disp = target - (pc as i64 + 2);
    let (lo, hi) = match bits {
        8 => (-128i64, 127i64),
        _ => (-32768i64, 32767i64),
    };
    if bits == 8 && madmac && disp == 0 {
        return Err(asm
            .make_user_error("illegal bra.s with zero offset", node, true)
            .into());
    }
    if !(lo..=hi).contains(&disp) {
        if asm.opts.ignore_relative_offset_errors {
            return Ok(0);
        }
        return Err(asm
            .make_user_error(format!("branch out of range ({disp} bytes)"), node, true)
            .into());
    }
    Ok(disp as i16)
}

/// The movem register mask: runs of (class, first, last) -> bits.
fn movem_mask(list: &[(u8, u8, u8)], predec: bool) -> u16 {
    let mut mask = 0u16;
    for &(class, first, last) in list {
        let (lo, hi) = (first.min(last), first.max(last));
        for n in lo..=hi {
            let bit = if class == 0 {
                n as u16 // D0..D7 -> bits 0..7
            } else {
                8 + n as u16 // A0..A7 -> bits 8..15
            };
            mask |= 1 << bit;
        }
    }
    if predec {
        mask.reverse_bits()
    } else {
        mask
    }
}

/// Collect the extension words for one EA operand. Extension words are
/// deferred so they are written after the opcode word; the order of
/// `ext.push` calls must mirror the sizer's operand order.
fn write_ea_ext(
    asm: &mut Assembler,
    node: AstNodeRef,
    ea: &Ea68000,
    size: Size,
    _pc: usize,
    scope: u64,
    child: &mut usize,
    madmac: bool,
    ext: &mut Vec<u16>,
) -> GResult<()> {
    match *ea {
        Ea68000::Dn(_)
        | Ea68000::An(_)
        | Ea68000::AnInd(_)
        | Ea68000::AnPostInc(_)
        | Ea68000::AnPreDec(_)
        | Ea68000::Sr
        | Ea68000::Ccr => Ok(()),

        Ea68000::Disp16(_) => {
            let v = eval_child(asm, node, scope, child)
                .ok_or_else(|| asm.make_user_error("missing displacement", node, true))?;
            ext.push(v as u16);
            Ok(())
        }

        Ea68000::DispIdx {
            idx,
            is_a,
            scale,
            long,
            ..
        } => {
            let v = eval_child(asm, node, scope, child)
                .ok_or_else(|| asm.make_user_error("missing displacement", node, true))?;
            if !(-128..=127).contains(&v) {
                return Err(asm
                    .make_user_error(
                        format!("indexed displacement out of range ({v})"),
                        node,
                        true,
                    )
                    .into());
            }
            // Real 68000 index word (M68000PRM/MAME):
            //   D/A<<15 | reg<<12 | long<<11 | scale<<9 | d8
            let w = (if is_a { 1 } else { 0 }) << 15
                | (idx as u16) << 12
                | (if long { 1 } else { 0 }) << 11
                | (scale as u16) << 9
                | (v as u16 & 0xff);
            ext.push(w);
            Ok(())
        }

        Ea68000::AbsW => {
            let v = eval_child(asm, node, scope, child)
                .ok_or_else(|| asm.make_user_error("missing absolute address", node, true))?;
            if madmac && (v + 0x8000) >= 0x10000 {
                return Err(asm
                    .make_user_error(format!("absolute .w out of range ({v:#x})"), node, true)
                    .into());
            }
            ext.push(v as u16);
            Ok(())
        }

        Ea68000::AbsL => {
            let v = eval_child(asm, node, scope, child)
                .ok_or_else(|| asm.make_user_error("missing absolute address", node, true))?;
            ext.push(((v as u32) >> 16) as u16);
            ext.push(v as u16);
            Ok(())
        }

        Ea68000::PcDisp16 => {
            // the operand is the raw displacement (the disassembler
            // renders the extension word; labelled sources give targets)
            let v = eval_child(asm, node, scope, child).ok_or_else(|| {
                asm.make_user_error("missing pc-relative displacement", node, true)
            })?;
            ext.push(v as u16);
            Ok(())
        }

        Ea68000::PcDispIdx {
            idx,
            is_a,
            scale,
            long,
        } => {
            let v = eval_child(asm, node, scope, child).ok_or_else(|| {
                asm.make_user_error("missing pc-relative displacement", node, true)
            })?;
            if !(-128..=127).contains(&v) {
                return Err(asm
                    .make_user_error(
                        format!("pc-relative displacement out of range ({v})"),
                        node,
                        true,
                    )
                    .into());
            }
            // Real 68000 index word (M68000PRM/MAME):
            //   D/A<<15 | reg<<12 | long<<11 | scale<<9 | d8
            let w = (if is_a { 1 } else { 0 }) << 15
                | (idx as u16) << 12
                | (if long { 1 } else { 0 }) << 11
                | (scale as u16) << 9
                | (v as u16 & 0xff);
            ext.push(w);
            Ok(())
        }

        Ea68000::Imm => {
            let v = eval_child(asm, node, scope, child)
                .ok_or_else(|| asm.make_user_error("missing immediate", node, true))?;
            match size {
                Size::B => ext.push((v & 0xff) as u16),
                Size::L => {
                    ext.push(((v as u32) >> 16) as u16);
                    ext.push(v as u16);
                }
                _ => ext.push((v & 0xffff) as u16),
            }
            Ok(())
        }

        Ea68000::Abs => Err(asm
            .make_user_error("internal: unresolved bare absolute", node, true)
            .into()),
    }
}

/// Compile a node (registry entry: `crate::cpu68000::assembler::compile_node`).
pub fn compile_node(
    asm: &mut Assembler,
    node: AstNodeRef,
    node_kind: NodeKind68000,
    current_scope_id: u64,
) -> GResult<()> {
    if let OpCode(row, ins) = node_kind {
        let insn = table().row(row);
        let pc = asm.get_binary().get_write_address();
        let madmac = asm.opts.madmac_compat;
        let mut word = insn.pattern;
        let mut ext: Vec<u16> = Vec::new();
        let mut child = 0usize;

        // op1/op2 helpers
        let ea1 = match &ins.op1 {
            Some(Operand68000::Ea(ea)) => Some(*ea),
            _ => Option::None,
        };
        let ea2 = match &ins.op2 {
            Some(Operand68000::Ea(ea)) => Some(*ea),
            _ => Option::None,
        };
        let _ = &ea1;
        let reg1 = |ea: &Ea68000| -> u16 {
            match ea {
                Ea68000::Dn(n) | Ea68000::An(n) => (*n as u16) << 9,
                Ea68000::AnPreDec(n) | Ea68000::AnPostInc(n) => (*n as u16) << 9,
                _ => 0,
            }
        };
        let reg2 = |ea: &Ea68000| -> u16 {
            match ea {
                Ea68000::Dn(n) | Ea68000::An(n) => *n as u16,
                Ea68000::AnPreDec(n) | Ea68000::AnPostInc(n) => *n as u16,
                _ => 0,
            }
        };

        use Form::*;
        match insn.form {
            RegEa => {
                if let Some(ea) = &ea1 {
                    word |= reg1(ea);
                }
                if let Some(ea) = &ea2 {
                    word |= ea_bits(ea);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
            }
            EaReg => {
                if let Some(ea) = &ea1 {
                    word |= ea_bits(ea);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
                if let Some(ea) = &ea2 {
                    word |= reg1(ea);
                }
            }
            EaAddr => {
                if let Some(ea) = &ea1 {
                    word |= ea_bits(ea);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
                if let Some(ea) = &ea2 {
                    word |= reg1(ea);
                }
            }
            RegReg | Exg => {
                if let Some(ea) = &ea2 {
                    word |= reg1(ea);
                }
                if let Some(ea) = &ea1 {
                    word |= reg2(ea);
                }
            }
            PredecPredec | PostincPostinc => {
                if let Some(ea) = &ea2 {
                    word |= reg1(ea);
                }
                if let Some(ea) = &ea1 {
                    word |= reg2(ea);
                }
            }
            ImmEa => {
                if let Some(ea) = &ea1 {
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
                if let Some(ea) = &ea2 {
                    word |= ea_bits(ea);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
            }
            BitImmEa => {
                // The bit-immediate is a full extension word; the opcode's
                // bit field reproduces the disassembler's rendering
                // (bit = (imm >> 8) & 7), and the executor reads the bit
                // from the extension word.
                if let Some(Ea68000::Imm) = ea1 {
                    if let Some(v) = eval_child(asm, node, current_scope_id, &mut child) {
                        word |= (((v >> 8) & 7) as u16) << 6;
                        ext.push((v & 0xffff) as u16);
                    }
                }
                if let Some(ea) = &ea2 {
                    word |= ea_bits(ea);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
            }
            ImmOnly => {
                if let Some(ea) = &ea1 {
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
            }
            Move => {
                if let Some(ea) = &ea1 {
                    word |= ea_bits(ea);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
                if let Some(ea) = &ea2 {
                    let (mode, reg) = ea_mode_reg(ea);
                    word |= ((mode as u16) << 6) | ((reg as u16) << 9);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
            }
            Moveq => {
                if let Some(Ea68000::Imm) = ea1 {
                    if let Some(v) = eval_child(asm, node, current_scope_id, &mut child) {
                        if !(-128..=127).contains(&v) {
                            return Err(asm
                                .make_user_error(
                                    format!("moveq immediate out of range ({v})"),
                                    node,
                                    true,
                                )
                                .into());
                        }
                        word |= (v as u16) & 0xff;
                    }
                }
                if let Some(ea) = &ea2 {
                    word |= reg1(ea);
                }
            }
            Addq | ShiftImm => {
                if let Some(Ea68000::Imm) = ea1 {
                    if let Some(v) = eval_child(asm, node, current_scope_id, &mut child) {
                        if !(1..=8).contains(&v) {
                            return Err(asm
                                .make_user_error(
                                    format!("quick immediate out of range ({v})"),
                                    node,
                                    true,
                                )
                                .into());
                        }
                        let q = if v == 8 { 0 } else { v as u16 };
                        word |= q << 9;
                    }
                }
                if insn.form == ShiftImm {
                    if let Some(ea) = &ea2 {
                        word |= reg2(ea);
                    }
                } else if let Some(ea) = &ea2 {
                    word |= ea_bits(ea);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
            }
            ShiftReg => {
                if let Some(ea) = &ea1 {
                    word |= reg1(ea);
                }
                if let Some(ea) = &ea2 {
                    word |= reg2(ea);
                }
            }
            ShiftEa | Scc | Ea => {
                if let Some(ea) = &ea1 {
                    word |= ea_bits(ea);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
            }
            Bcc8 => {
                if let Some(target) = eval_child(asm, node, current_scope_id, &mut child) {
                    let d = rel_disp(asm, node, target, pc, 8, madmac)?;
                    word |= (d as u16) & 0xff;
                }
            }
            Bcc16 => {
                if let Some(target) = eval_child(asm, node, current_scope_id, &mut child) {
                    let d = rel_disp(asm, node, target, pc, 16, madmac)?;
                    ext.push(d as u16);
                }
            }
            Dbcc => {
                if let Some(ea) = &ea1 {
                    word |= reg2(ea);
                }
                if let Some(target) = eval_child(asm, node, current_scope_id, &mut child) {
                    let d = rel_disp(asm, node, target, pc, 16, madmac)?;
                    ext.push(d as u16);
                }
            }
            MovemRe | MovemPd | MovemEr => {
                let predec = insn.form == MovemPd;
                let empty: Vec<(u8, u8, u8)> = vec![];
                let (list, ea) = match (&ins.op1, &ins.op2) {
                    (Some(Operand68000::Movem(list)), Some(Operand68000::Ea(ea))) => {
                        (list, Some(*ea))
                    }
                    (Some(Operand68000::Ea(ea)), Some(Operand68000::Movem(list))) => {
                        (list, Some(*ea))
                    }
                    (Some(Operand68000::Movem(list)), Option::None) => (list, Option::None),
                    _ => (&empty, Option::None),
                };
                ext.push(movem_mask(&list, predec));
                if let Some(ea) = &ea {
                    word |= ea_bits(ea);
                    write_ea_ext(
                        asm,
                        node,
                        ea,
                        insn.size,
                        pc,
                        current_scope_id,
                        &mut child,
                        madmac,
                        &mut ext,
                    )?;
                }
            }
            Link => {
                if let Some(ea) = &ea1 {
                    word |= reg2(ea);
                }
                if let Some(Ea68000::Imm) = ea2 {
                    if let Some(v) = eval_child(asm, node, current_scope_id, &mut child) {
                        ext.push(v as u16);
                    }
                }
            }
            Movep => {
                // bits 11-9 = Dn, bits 2-0 = An
                let (dn, an, disp_first) = match (ea1, ea2) {
                    (Some(Ea68000::Dn(n)), Some(Ea68000::Disp16(a))) => (n, a, false),
                    (Some(Ea68000::Disp16(a)), Some(Ea68000::Dn(n))) => (n, a, true),
                    _ => (0, 0, false),
                };
                word |= ((dn as u16) << 9) | (an as u16);
                let v = eval_child(asm, node, current_scope_id, &mut child).unwrap_or(0);
                ext.push(v as u16);
                let _ = disp_first;
            }
            Trap => {
                if let Some(Ea68000::Imm) = ea1 {
                    if let Some(v) = eval_child(asm, node, current_scope_id, &mut child) {
                        if !(0..=15).contains(&v) {
                            return Err(asm
                                .make_user_error(
                                    format!("trap vector out of range ({v})"),
                                    node,
                                    true,
                                )
                                .into());
                        }
                        word |= (v as u16) & 0xf;
                    }
                }
            }
            Dreg | Areg => {
                if let Some(ea) = &ea1 {
                    word |= reg2(ea);
                }
            }
            Imm16 => {
                if let Some(Ea68000::Imm) = ea1 {
                    if let Some(v) = eval_child(asm, node, current_scope_id, &mut child) {
                        ext.push(v as u16);
                    }
                }
            }
            None => {}
        }

        // opcode word first, then the extension words in order
        asm.write_word(word, node)?;
        for w in ext {
            asm.write_word(w, node)?;
        }
        return Ok(());
    }
    panic!("compile_node: unexpected node kind")
}
