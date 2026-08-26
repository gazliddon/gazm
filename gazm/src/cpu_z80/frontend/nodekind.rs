#![forbid(unused_imports)]
use crate::frontend::CpuSpecific;
use emuz80::isa::InstructionId;

/// 8-bit registers. `IXH`/`IXL`/`IYH`/`IYL` are the undocumented halves of
/// IX/IY; they select the DD/FD prefixed encodings of `r`-forms.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Reg8 {
    B,
    C,
    D,
    E,
    H,
    L,
    A,
    IXH,
    IXL,
    IYH,
    IYL,
}

impl Reg8 {
    pub fn from_text(text: &str) -> Option<Self> {
        use Reg8::*;
        Some(match text {
            "b" => B,
            "c" => C,
            "d" => D,
            "e" => E,
            "h" => H,
            "l" => L,
            "a" => A,
            "ixh" => IXH,
            "ixl" => IXL,
            "iyh" => IYH,
            "iyl" => IYL,
            _ => return None,
        })
    }

    /// The `r` field value (used with the row's `bit_fields` shift).
    pub fn bits(self) -> u8 {
        use Reg8::*;
        match self {
            B => 0,
            C => 1,
            D => 2,
            E => 3,
            H => 4,
            L => 5,
            A => 7,
            IXH => 4,
            IXL => 5,
            IYH => 4,
            IYL => 5,
        }
    }

    /// The opcode prefix this register requires: DD for the IX halves,
    /// FD for the IY halves, none otherwise.
    pub fn prefix(self) -> Option<u16> {
        use Reg8::*;
        match self {
            IXH | IXL => Some(0xDD00),
            IYH | IYL => Some(0xFD00),
            _ => None,
        }
    }
}

/// 16-bit register pairs. `AF` is deliberately absent — it has its own
/// literal templates (POP AF, PUSH AF) and is not a `dd` operand.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Pair {
    BC,
    DE,
    HL,
    SP,
    IX,
    IY,
}

impl Pair {
    pub fn from_text(text: &str) -> Option<Self> {
        use Pair::*;
        Some(match text {
            "bc" => BC,
            "de" => DE,
            "hl" => HL,
            "sp" => SP,
            "ix" => IX,
            "iy" => IY,
            _ => return None,
        })
    }

    /// The `dd` field value (used with the row's `bit_fields` shift).
    pub fn bits(self) -> u8 {
        use Pair::*;
        match self {
            BC => 0,
            DE => 1,
            HL => 2,
            SP => 3,
            IX => 2,
            IY => 2,
        }
    }
}

/// How an instruction's parsed operands are carried in the AST: which
/// register/bit/vector values to OR into the opcode byte, and how many
/// expression children follow (evaluated in order: d then n).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperandParseType {
    None,
    /// One expression child (n/nn/e/d).
    Expr,
    /// Two expression children: the `(IX+d)` displacement then `n`.
    ExprExpr,
    Reg(Reg8),
    RegExpr(Reg8),
    RegReg(Reg8, Reg8),
    /// `r,(HL)` — register bits only.
    RegIndirect(Reg8),
    /// `(HL),r` — register bits only.
    IndirectReg(Reg8),
    /// `r,(IX+d)` — register bits + one d child.
    RegIndexed(Reg8),
    /// `(IX+d),r` — register bits + one d child.
    IndexedReg(Reg8),
    Pair(Pair),
    PairExpr(Pair),
    /// `b,(HL)` — bit number only.
    BitIndirect(u8),
    /// `b,(IX+d)` — bit number + one d child.
    BitIndexed(u8),
    /// `b,r` — bit number + register.
    BitReg(u8, Reg8),
    /// `b,(IX+d),r` — bit number + register + one d child.
    BitIndexedReg(u8, Reg8),
    /// RST vector (0-7).
    Restart(u8),
}

/// The value a `bit_fields` variable takes for this operand: `r`/`r1`/`r2`
/// -> register field, `dd` -> pair field, `b` -> bit number, `p` -> vector.
pub fn field_value(var: &str, op: &OperandParseType) -> Option<u8> {
    use OperandParseType::{
        BitIndexed, BitIndexedReg, BitIndirect, BitReg, IndexedReg, IndirectReg, Pair, PairExpr,
        Reg, RegExpr, RegIndexed, RegIndirect, RegReg, Restart,
    };
    match var {
        "r" => match op {
            Reg(r)
            | RegExpr(r)
            | RegReg(r, _)
            | RegIndirect(r)
            | IndirectReg(r)
            | RegIndexed(r)
            | IndexedReg(r)
            | BitReg(_, r)
            | BitIndexedReg(_, r) => Some(r.bits()),
            _ => None,
        },
        "r1" => match op {
            RegReg(r1, _) => Some(r1.bits()),
            _ => None,
        },
        "r2" => match op {
            RegReg(_, r2) => Some(r2.bits()),
            _ => None,
        },
        "dd" => match op {
            Pair(p) | PairExpr(p) => Some(p.bits()),
            _ => None,
        },
        "b" => match op {
            BitReg(b, _) | BitIndirect(b) | BitIndexed(b) | BitIndexedReg(b, _) => Some(*b),
            _ => None,
        },
        "p" => match op {
            Restart(p) => Some(*p),
            _ => None,
        },
        _ => None,
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum NodeKindZ80 {
    #[default]
    Illegal,
    OpCode(InstructionId, OperandParseType),
}

impl From<NodeKindZ80> for CpuSpecific {
    fn from(value: NodeKindZ80) -> Self {
        CpuSpecific::CpuZ80(value)
    }
}

impl From<NodeKindZ80> for crate::cpu_z80::NodeKind {
    fn from(value: NodeKindZ80) -> Self {
        crate::cpu_z80::NodeKind::TargetSpecific(value.into())
    }
}
