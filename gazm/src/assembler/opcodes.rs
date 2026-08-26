//! Row-driven opcode emission shared by the CPU backends.
//!
//! The per-CPU compile paths resolve an instruction row, write its opcode
//! value, then its operands. These writers are the common core; each
//! backend supplies its own mode -> operand mapping plus hooks for the
//! genuinely CPU-specific forms (the 6809's indexed postbytes and
//! register sets, the Z80's template/bit-field encoding, ...).

use super::{binary::BinaryError, Assembler};
use crate::error::GResult;
use crate::semantic::AstNodeRef;

/// Write the opcode value: one byte, or a word for prefixed opcodes
/// (0x10xx on the 6809, 0xEDxx on the Z80).
pub fn write_opcode(asm: &mut Assembler, node: AstNodeRef, opcode: usize) -> GResult<()> {
    if opcode > 0xff {
        asm.write_word(opcode as u16, node)
    } else {
        asm.write_byte(opcode as u8, node)
    }
}

/// Evaluate the node's first argument and write it as a byte operand.
pub fn write_eval_byte(
    asm: &mut Assembler,
    node: AstNodeRef,
    current_scope_id: u64,
) -> GResult<()> {
    let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
    asm.write_byte_check_size(arg, node)?;
    Ok(())
}

/// Evaluate the node's first argument and write it as a word operand.
pub fn write_eval_word(
    asm: &mut Assembler,
    node: AstNodeRef,
    current_scope_id: u64,
) -> GResult<()> {
    let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
    asm.write_word_check_size(arg, node)?;
    Ok(())
}

/// Write a signed byte operand (indexed displacements etc.).
pub fn write_signed_byte(asm: &mut Assembler, node: AstNodeRef, val: i64) -> GResult<()> {
    let res = asm.get_binary_mut().write_ibyte_check_size(val);
    asm.binary_error_map(node, res)?;
    Ok(())
}

/// Branch displacement: `target - (address after the instruction)`.
/// Out-of-range offsets honour `ignore_relative_offset_errors` by writing
/// 0 instead of failing.
pub fn write_relative_byte(
    asm: &mut Assembler,
    node: AstNodeRef,
    target: i64,
    pc: usize,
    size: usize,
) -> GResult<()> {
    let val = target - (pc as i64 + size as i64);

    match asm.get_binary_mut().write_ibyte_check_size(val) {
        Ok(_) => Ok(()),
        Err(BinaryError::DoesNotFit { .. }) if asm.opts.ignore_relative_offset_errors => {
            let res = asm.get_binary_mut().write_ibyte_check_size(0);
            asm.binary_error_map(node, res)?;
            Ok(())
        }
        Err(BinaryError::DoesNotFit { .. }) => {
            let message = if val < 0 {
                format!("Branch out of range by {} bytes ({val})", (128 + val).abs())
            } else {
                format!("Branch out of range by {} bytes ({val})", val - 127)
            };
            Err(asm.make_user_error(message, node, true).into())
        }
        Err(e) => asm.binary_error_map(node, Err(e)),
    }
}

/// 16-bit branch displacement: same base as [`write_relative_byte`],
/// wrapped to 16 bits (the 6809's Relative16 form).
pub fn write_relative_word(
    asm: &mut Assembler,
    node: AstNodeRef,
    target: i64,
    pc: usize,
    size: usize,
) -> GResult<()> {
    let val = (target - (pc as i64 + size as i64)) & 0xffff;
    let res = asm.get_binary_mut().write_word_check_size(val);
    asm.binary_error_map(node, res)?;
    Ok(())
}

/// Direct-page optimisation: when the instruction row supports a direct
/// mode and the operand's high byte equals the configured direct page,
/// invoke `make_fixup` with the direct row's id and return its size.
///
/// `value` is the pre-evaluated operand; None (unresolved forward
/// reference, or the instruction has no operand) skips the optimisation.
/// `get_direct` resolves the direct-mode row in the backend's own
/// database. Returns None when the optimisation does not apply.
pub fn try_direct_page(
    dp: Option<u8>,
    value: Option<i64>,
    get_direct: impl FnOnce() -> Option<(usize, usize)>,
    make_fixup: impl FnOnce(usize),
) -> Option<usize> {
    let dp = dp?;
    let (new_size, new_id) = get_direct()?;
    let value = value?;
    if ((value >> 8) & 0xff) as u8 == dp {
        make_fixup(new_id);
        Some(new_size)
    } else {
        None
    }
}
