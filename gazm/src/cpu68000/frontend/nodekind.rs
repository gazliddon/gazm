#![deny(unused_imports)]
use crate::frontend::CpuSpecific;

/// One parsed effective-address operand, MadMac style. Expression-bearing
/// forms (displacements, absolutes, immediates, branch targets) carry the
/// expression as a child node of the `OpCode` node, in operand order.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Ea68000 {
    Dn(u8),
    An(u8),
    AnInd(u8),
    AnPostInc(u8),
    AnPreDec(u8),
    /// d16(An): one extension word; expression child 0.
    Disp16(u8),
    /// d8(An,Xn): one extension word (D/A<<15 | reg<<12 | long<<11 |
    /// scale<<9 | d8); expression child 0.
    DispIdx {
        an: u8,
        idx: u8,
        is_a: bool,
        scale: u8,
        long: bool,
    },
    /// $xxxx.w — one extension word; expression child 0.
    AbsW,
    /// $xxxx.l — two extension words; expression child 0.
    AbsL,
    /// d16(PC): one extension word (relative to the ext word + 2);
    /// expression child 0.
    PcDisp16,
    /// d8(PC,Xn): one extension word (D/A<<15 | reg<<12 | long<<11 |
    /// scale<<9 | d8); expression child 0.
    PcDispIdx {
        idx: u8,
        is_a: bool,
        scale: u8,
        long: bool,
    },
    /// #expr — 1/2 words by operation size; expression child 0.
    Imm,
    /// The special `sr` register operand (move.w sr,<ea> / move.w <ea>,sr).
    Sr,
    /// The special `ccr` register operand (move.w ccr,<ea> / move.w <ea>,ccr).
    Ccr,
    /// Bare expression: absolute, width chosen at size time (MadMac:
    /// long by default, optimized to short when the value is a defined
    /// absolute that fits signed 16 bits; expression child 0).
    Abs,
}

/// A movem register list: runs of `(class, first, last)` in source order.
pub type MovemList = Vec<(u8, u8, u8)>;

/// The two parsed operands of an instruction, plus the chosen table row.
/// Expression children are attached to the `OpCode` node in order: op1's
/// expression (if any), then op2's.
#[derive(Debug, PartialEq, Clone)]
pub struct Instr68000 {
    pub op1: Option<Operand68000>,
    pub op2: Option<Operand68000>,
    /// Explicit `.s` on a branch: force the 8-bit form (no auto-extend,
    /// zero/range displacements error at compile time).
    pub short: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand68000 {
    Ea(Ea68000),
    Movem(MovemList),
}

impl Operand68000 {
    /// Does this operand carry an expression child?
    pub fn has_expr(&self) -> bool {
        matches!(
            self,
            Operand68000::Ea(
                Ea68000::Disp16(_)
                    | Ea68000::DispIdx { .. }
                    | Ea68000::AbsW
                    | Ea68000::AbsL
                    | Ea68000::PcDisp16
                    | Ea68000::PcDispIdx { .. }
                    | Ea68000::Imm
                    | Ea68000::Abs
            )
        )
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum NodeKind68000 {
    #[default]
    Illegal,
    /// The resolved table row index plus the parsed operands.
    OpCode(usize, Instr68000),
    Operand(Ea68000),
}

impl From<NodeKind68000> for CpuSpecific {
    fn from(value: NodeKind68000) -> Self {
        CpuSpecific::Cpu68000(value)
    }
}

impl From<NodeKind68000> for crate::frontend::AstNodeKind {
    fn from(value: NodeKind68000) -> Self {
        crate::frontend::AstNodeKind::TargetSpecific(CpuSpecific::Cpu68000(value))
    }
}
