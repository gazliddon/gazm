use crate::cpu_z80::assembler::ISA_DBASE;
use crate::cpu_z80::frontend::{field_value, NodeKindZ80, OperandParseType};
use crate::{
    assembler::{Assembler, BinaryError},
    error::GResult,
    semantic::AstNodeRef,
};

use emuz80::isa::Instruction;

/// Compile a Z80 node: write the opcode bytes (with register/bit/vector
/// fields ORed in) then the operand bytes.
pub fn compile_node(
    asm: &mut Assembler,
    node: AstNodeRef,
    node_kind: NodeKindZ80,
    current_scope_id: u64,
) -> GResult<()> {
    match node_kind {
        NodeKindZ80::OpCode(id, op) => {
            let ins = ISA_DBASE.get_by_id(id).expect("stale instruction id");
            compile_opcode(asm, node, ins, op, current_scope_id)?;
        }
        NodeKindZ80::Illegal => panic!(),
    }
    Ok(())
}

fn compile_opcode(
    asm: &mut Assembler,
    node: AstNodeRef,
    ins: &Instruction,
    op: OperandParseType,
    current_scope_id: u64,
) -> GResult<()> {
    // PC at the start of the instruction: relative operands are measured
    // from the address after the whole instruction.
    let pc = asm.get_binary().get_write_address();

    // Opcode value with the symbolic fields ORed into the low byte.
    let mut opval = ins.opcode as u64;
    let mut low = (opval & 0xff) as u8;
    for (var, shift) in &ins.bit_fields {
        let value = field_value(var, &op).ok_or_else(|| {
            asm.make_user_error(format!("internal: missing bit field {var}"), node, true)
        })?;
        low |= value << shift;
    }
    opval = (opval & !0xff) | low as u64;

    // Physical byte order: most opcodes are prefix(es), opcode, operands —
    // but the DD/FD CB d forms interleave (`DD CB <d> <op>`), so write the
    // opcode bytes before and after the operands separately.
    let opcode_bytes = ins.size - ins.operand_size;
    // Number of opcode bytes physically before the operand bytes. Zero for
    // every form except the DD/FD CB d family (DD CB <d> <op>).
    let operands_at = ins.operand_offset.min(opcode_bytes);
    let before = opcode_bytes - operands_at;
    for i in (before..opcode_bytes).rev() {
        asm.write_byte(((opval >> (8 * i)) & 0xff) as u8, node)?;
    }

    use OperandParseType::*;
    match op {
        None | Reg(_) | RegReg(..) | RegIndirect(_) | IndirectReg(_) | Pair(_) | BitIndirect(_)
        | BitReg(..) | Restart(_) => {}

        Expr => write_operand(asm, node, ins, pc, current_scope_id)?,

        // (IX+d),n — displacement then value.
        ExprExpr => {
            let (d, n) = asm.eval_two_args(node, current_scope_id)?;
            write_signed_byte(asm, node, d)?;
            asm.write_byte_check_size(n, node)?;
        }

        RegExpr(_) | RegIndexed(_) | IndexedReg(_) | PairExpr(_) | BitIndexed(_)
        | BitIndexedReg(..) => write_operand(asm, node, ins, pc, current_scope_id)?,
    }

    // Trailing opcode bytes (the DD/FD CB operation byte after the d).
    for i in (0..before).rev() {
        asm.write_byte(((opval >> (8 * i)) & 0xff) as u8, node)?;
    }
    Ok(())
}

/// Write the single expression operand, sized by the instruction's mode.
fn write_operand(
    asm: &mut Assembler,
    node: AstNodeRef,
    ins: &Instruction,
    pc: usize,
    current_scope_id: u64,
) -> GResult<()> {
    use emuz80::isa::AddrModeEnum::*;
    let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;

    match ins.addr_mode {
        Immediate8 | Port | Indirect => asm.write_byte_check_size(arg, node)?,
        Immediate16 | AbsoluteIndirect | AbsoluteIndirect16 | ConditionImmediate => {
            asm.write_word_check_size(arg, node)?
        }
        Indexed | BitIndexed => write_signed_byte(asm, node, arg)?,
        Relative | ConditionRelative => write_relative(asm, node, ins, pc, arg)?,
        _ => {
            return Err(asm
                .make_user_error("internal: unexpected operand mode", node, true)
                .into())
        }
    }
    Ok(())
}

fn write_signed_byte(asm: &mut Assembler, node: AstNodeRef, val: i64) -> GResult<()> {
    let res = asm.get_binary_mut().write_ibyte_check_size(val);
    asm.binary_error_map(node, res)?;
    Ok(())
}

/// JR/DJNZ style displacement: target - (address after the instruction).
fn write_relative(
    asm: &mut Assembler,
    node: AstNodeRef,
    ins: &Instruction,
    pc: usize,
    target: i64,
) -> GResult<()> {
    let val = target - (pc as i64 + ins.size as i64);

    match asm.get_binary_mut().write_ibyte_check_size(val) {
        Ok(_) => Ok(()),
        Err(BinaryError::DoesNotFit { .. }) => {
            if asm.opts.ignore_relative_offset_errors {
                let res = asm.get_binary_mut().write_ibyte_check_size(0);
                asm.binary_error_map(node, res)?;
                Ok(())
            } else {
                Err(asm
                    .make_user_error(
                        format!("Relative jump out of range ({val} bytes)"),
                        node,
                        true,
                    )
                    .into())
            }
        }
        Err(e) => asm.binary_error_map(node, Err(e)),
    }
}
