#![deny(unused_imports)]
use crate::assembler::{Assembler, Sizer};
use crate::cpu68000::{
    frontend::{Ea68000, Instr68000, NodeKind68000, Operand68000},
    isa::table,
};
use crate::error::GResult;
use crate::semantic::AstNodeId;

use emu68000::isa::{Form, Size};

use NodeKind68000::OpCode;

/// Extension words for one EA operand.
fn ea_ext_words(ea: &Ea68000, size: Size) -> usize {
    match ea {
        Ea68000::Dn(_)
        | Ea68000::An(_)
        | Ea68000::AnInd(_)
        | Ea68000::AnPostInc(_)
        | Ea68000::AnPreDec(_)
        | Ea68000::Sr
        | Ea68000::Ccr => 0,
        Ea68000::Disp16(_)
        | Ea68000::DispIdx { .. }
        | Ea68000::AbsW
        | Ea68000::PcDisp16
        | Ea68000::PcDispIdx { .. } => 1,
        Ea68000::AbsL => 2,
        Ea68000::Imm => match size {
            Size::L => 2,
            _ => 1,
        },
        Ea68000::Abs => 0, // resolved below
    }
}

/// Size a 68000 node (registry entry: `crate::cpu68000::assembler::size_node_internal`).
pub fn size_node_internal(
    sizer: &mut Sizer,
    asm: &mut Assembler,
    id: AstNodeId,
    node_kind: NodeKind68000,
) -> GResult<()> {
    let current_scope_id = sizer.scopes.scope();
    if let OpCode(row, ins) = node_kind {
        let insn = table().row(row);

        // Branches: the displacement is never an EA extension word.
        // Bare branches (MadMac m_br): with a defined target, use the
        // 8-bit form when `v != 0 && v+0x80 < 0x100` (v = target-(pc+2)),
        // otherwise auto-extend to the 16-bit form; forward references
        // stay 16-bit. An explicit `.s`/`.b` forces 8-bit (no extend).
        match insn.form {
            Form::Bcc8 => {
                if ins.short {
                    sizer.advance_pc(2);
                    return Ok(());
                }
                let node = sizer.get_node(id);
                if let Ok((target, _)) = asm.eval_first_arg(node, current_scope_id) {
                    // MadMac computes v in 32-bit unsigned arithmetic, so
                    // the range check is unsigned: -128..127 (minus zero).
                    let v = target - (sizer.get_pc() as i64 + 2);
                    if v != 0 && ((v + 0x80) as u64) < 0x100 {
                        sizer.advance_pc(2);
                        return Ok(());
                    }
                }
                // out of range or forward: widen to the 16-bit form
                if let Some(wrow) = table().branch_w_row(insn.mnemonic) {
                    sizer.set_node_fixup(
                        id,
                        OpCode(
                            wrow,
                            Instr68000 {
                                op1: ins.op1.clone(),
                                op2: ins.op2.clone(),
                                short: false,
                            },
                        ),
                    );
                }
                sizer.advance_pc(4);
                return Ok(());
            }
            Form::Bcc16 | Form::Dbcc => {
                sizer.advance_pc(4);
                return Ok(());
            }
            // MadMac OPT_MOVEL_MOVEQ: `move.l #imm,Dn` with a defined
            // immediate that fits signed 8 bits assembles as `moveq`
            // (forward references stay `move.l`, exactly like rmac).
            Form::Move if insn.size == Size::L => {
                if let (
                    Some(Operand68000::Ea(Ea68000::Imm)),
                    Some(Operand68000::Ea(Ea68000::Dn(_))),
                ) = (&ins.op1, &ins.op2)
                {
                    let node = sizer.get_node(id);
                    if let Ok((v, _)) = asm.eval_first_arg(node, current_scope_id) {
                        if ((v + 0x80) as u64) < 0x100 {
                            if let Some(qrow) = table().lookup("moveq", None).and_then(|rows| {
                                rows.into_iter()
                                    .find(|&i| table().row(i).form == Form::Moveq)
                            }) {
                                sizer.set_node_fixup(
                                    id,
                                    OpCode(
                                        qrow,
                                        Instr68000 {
                                            op1: ins.op1.clone(),
                                            op2: ins.op2.clone(),
                                            short: false,
                                        },
                                    ),
                                );
                                sizer.advance_pc(2);
                                return Ok(());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        let mut new_ins = ins.clone();
        let mut size = 2;
        let mut fixup = false;
        // forms whose immediate lives in the opcode word, not an
        // extension word
        let imm_in_word = matches!(
            insn.form,
            Form::Addq | Form::ShiftImm | Form::Moveq | Form::Trap
        );

        // Resolve bare-absolute widths (MadMac: long by default, short
        // only for a defined absolute that fits signed 16 bits; forward
        // references stay long) and count extension words.
        let mut child = 0usize;
        for op in [&mut new_ins.op1, &mut new_ins.op2] {
            let mut has_expr = false;
            if let Some(Operand68000::Ea(Ea68000::Abs)) = op {
                let node = sizer.get_node(id);
                let value = node
                    .children()
                    .nth(child)
                    .and_then(|c| asm.eval_node(c, current_scope_id).ok());
                let short = match value {
                    Some(v) => (v + 0x8000) < 0x10000, // signed 16 bits
                    None => false,                     // forward reference -> long
                };
                *op = Some(Operand68000::Ea(if short {
                    Ea68000::AbsW
                } else {
                    Ea68000::AbsL
                }));
                fixup = true;
            }
            if let Some(Operand68000::Ea(ea)) = op {
                if !(imm_in_word && matches!(ea, Ea68000::Imm)) {
                    size += 2 * ea_ext_words(ea, insn.size);
                }
                has_expr = Operand68000::Ea(*ea).has_expr();
            }
            if matches!(op, Some(Operand68000::Movem(_))) {
                size += 2; // the register mask word
            }
            if has_expr {
                child += 1;
            }
        }

        if fixup {
            sizer.set_node_fixup(id, OpCode(row, new_ins));
        }
        sizer.advance_pc(size);
        return Ok(());
    }
    panic!("size_node_internal: unexpected node kind")
}
