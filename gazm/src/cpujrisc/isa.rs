//! The Jaguar RISC (GPU/DSP, "JRISC") instruction vocabulary.
//!
//! Mirrors rmac's `riscasm.c` `roptbl` table: every mnemonic maps to an
//! operand *type* (how to parse operands) and a 6-bit *opcode* (bits 15-10
//! of the instruction word, with register fields reg1=bits 9-5, reg2=bits
//! 4-0). The GPU/DSP-only flags are recorded so the backend can reject an
//! opcode assembled for the wrong processor, though Tempest 2000's GPU
//! sources never use DSP-only opcodes.

/// How the operands of a JRISC instruction are parsed and encoded.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JriscType {
    /// No operands: `nop`.
    None,
    /// One register operand `Rd`: `abs, mirror, neg, not, pack, resmac,
    /// sat8, sat16, sat16s, sat24, sat32s, unpack`. reg1 comes from the
    /// opcode parameter itself (0, except `pack` which sets bit 6).
    One,
    /// Two register operands `Rs,Rd`: `add, addc, and, cmp, div, imacn,
    /// imult, imultn, moveta, movefa, mult, mmult, mtoi, normi, or, ror,
    /// sh, sha, sub, subc, xor`.
    Two,
    /// `#n,Rd` with n = -16..15: `cmpq`.
    Num15,
    /// `#n,Rd` with n = 0..31: `bclr, bset, btst, moveq`.
    Num31,
    /// `#n,Rd` with n = 1..32 (32 stored as 0): `addq, addqt, addqmod,
    /// subq, subqmod, subqt, sharq, shrq, rorq, rolq`. `shlq` additionally
    /// stores 32-n (SUB32).
    Num32,
    /// `#imm,Rd` — a 32-bit immediate in the next two words
    /// ((imm & 0xFFFF) first, then (imm >> 16)).
    Movei,
    /// `PC,Rd` (opcode 51) or `Rs,Rd` (opcode 34): `move`.
    Move,
    /// `(Rn),Rd` (41), `(R14+n),Rd` (43), `(R15+n),Rd` (44),
    /// `(R14+Rn),Rd` (58), `(R15+Rn),Rd` (59): `load`.
    Load,
    /// `(Rs),Rd`: `loadb, loadp, loadw`.
    Loadn,
    /// `Rs,(Rn)` (47), `Rs,(R14+n)` (49), `Rs,(R15+n)` (50),
    /// `Rs,(R14+Rn)` (60), `Rs,(R15+Rn)` (61): `store`.
    Store,
    /// `Rs,(Rd)`: `storeb, storep, storew`.
    Storen,
    /// `cc,n` — PC-relative, n = (target-(pc+2))/2 in words, -16..15: `jr`.
    Jr,
    /// `cc,(Rn)`: `jump`.
    Jump,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct JriscOp {
    pub name: &'static str,
    pub typ: JriscType,
    /// 6-bit opcode in bits 15-10; for `pack`, bit 6 of the parameter is
    /// stored into reg1 (see `One`).
    pub op: u16,
    /// GPU-only opcode.
    pub gpu_only: bool,
    /// DSP-only opcode.
    pub dsp_only: bool,
    /// `shlq`: store 32-n in reg1.
    pub sub32: bool,
}

const fn op(
    name: &'static str,
    typ: JriscType,
    op: u16,
    gpu_only: bool,
    dsp_only: bool,
    sub32: bool,
) -> JriscOp {
    JriscOp {
        name,
        typ,
        op,
        gpu_only,
        dsp_only,
        sub32,
    }
}

/// The full JRISC vocabulary, in rmac `roptbl` order.
pub static OPS: &[JriscOp] = &[
    op("add", JriscType::Two, 0, false, false, false),
    op("addc", JriscType::Two, 1, false, false, false),
    op("addq", JriscType::Num32, 2, false, false, false),
    op("addqt", JriscType::Num32, 3, false, false, false),
    op("sub", JriscType::Two, 4, false, false, false),
    op("subc", JriscType::Two, 5, false, false, false),
    op("subq", JriscType::Num32, 6, false, false, false),
    op("subqt", JriscType::Num32, 7, false, false, false),
    op("neg", JriscType::One, 8, false, false, false),
    op("and", JriscType::Two, 9, false, false, false),
    op("or", JriscType::Two, 10, false, false, false),
    op("xor", JriscType::Two, 11, false, false, false),
    op("not", JriscType::One, 12, false, false, false),
    op("btst", JriscType::Num31, 13, false, false, false),
    op("bset", JriscType::Num31, 14, false, false, false),
    op("bclr", JriscType::Num31, 15, false, false, false),
    op("mult", JriscType::Two, 16, false, false, false),
    op("imult", JriscType::Two, 17, false, false, false),
    op("imultn", JriscType::Two, 18, false, false, false),
    op("resmac", JriscType::One, 19, false, false, false),
    op("imacn", JriscType::Two, 20, false, false, false),
    op("div", JriscType::Two, 21, false, false, false),
    op("abs", JriscType::One, 22, false, false, false),
    op("sh", JriscType::Two, 23, false, false, false),
    op("shlq", JriscType::Num32, 24, false, false, true),
    op("shrq", JriscType::Num32, 25, false, false, false),
    op("sha", JriscType::Two, 26, false, false, false),
    op("sharq", JriscType::Num32, 27, false, false, false),
    op("ror", JriscType::Two, 28, false, false, false),
    op("rorq", JriscType::Num32, 29, false, false, false),
    op("rolq", JriscType::Num32, 29, false, false, false),
    op("cmp", JriscType::Two, 30, false, false, false),
    op("cmpq", JriscType::Num15, 31, false, false, false),
    op("sat8", JriscType::One, 32, true, false, false),
    op("subqmod", JriscType::Num32, 32, false, true, false),
    op("sat16", JriscType::One, 33, true, false, false),
    op("sat16s", JriscType::One, 33, false, true, false),
    op("moveq", JriscType::Num31, 35, false, false, false),
    op("moveta", JriscType::Two, 36, false, false, false),
    op("movefa", JriscType::Two, 37, false, false, false),
    op("movei", JriscType::Movei, 38, false, false, false),
    op("loadb", JriscType::Loadn, 39, false, false, false),
    op("loadw", JriscType::Loadn, 40, false, false, false),
    op("loadp", JriscType::Loadn, 42, true, false, false),
    op("sat32s", JriscType::One, 42, false, true, false),
    op("storeb", JriscType::Storen, 45, false, false, false),
    op("storew", JriscType::Storen, 46, false, false, false),
    op("storep", JriscType::Storen, 48, true, false, false),
    op("mirror", JriscType::One, 48, false, true, false),
    op("jump", JriscType::Jump, 52, false, false, false),
    op("jr", JriscType::Jr, 53, false, false, false),
    op("mmult", JriscType::Two, 54, false, false, false),
    op("mtoi", JriscType::Two, 55, false, false, false),
    op("normi", JriscType::Two, 56, false, false, false),
    op("nop", JriscType::None, 57, false, false, false),
    op("sat24", JriscType::One, 62, false, false, false),
    op("unpack", JriscType::One, 63, true, false, false),
    op("pack", JriscType::One, 63, true, false, false),
    op("addqmod", JriscType::Num32, 63, false, true, false),
    // `move`/`load`/`store` are the polymorphic spellings whose opcode is
    // decided by the operand form (see JriscType::Move/Load/Store).
    op("move", JriscType::Move, 0, false, false, false),
    op("load", JriscType::Load, 0, false, false, false),
    op("store", JriscType::Store, 0, false, false, false),
];

/// Look up a mnemonic (lowercase) in the JRISC table.
pub fn find(name: &str) -> Option<&'static JriscOp> {
    OPS.iter().find(|o| o.name == name)
}

/// The mnemonic vocabulary for identifier classification.
pub fn mnemonic_set() -> impl Iterator<Item = &'static str> {
    OPS.iter().map(|o| o.name)
}

/// Jaguar jump condition names (rmac `condname`) -> condition numbers.
pub static CONDS: &[(&str, u8)] = &[
    ("NZ", 1),
    ("Z", 2),
    ("NC", 4),
    ("NCNZ", 5),
    ("NCZ", 6),
    ("C", 8),
    ("CNZ", 9),
    ("CZ", 10),
    ("NN", 20),
    ("NNNZ", 21),
    ("NNZ", 22),
    ("N", 24),
    ("N_NZ", 25),
    ("N_Z", 26),
    ("T", 0),
    ("A", 0),
    ("NE", 1),
    ("EQ", 2),
    ("CC", 4),
    ("HS", 4),
    ("HI", 5),
    ("CS", 8),
    ("LO", 8),
    ("PL", 20),
    ("MI", 24),
    ("F", 31),
];

/// Look up a condition-code name (any case) -> number.
pub fn cond(name: &str) -> Option<u8> {
    let l = name.to_ascii_lowercase();
    CONDS
        .iter()
        .find(|(n, _)| n.to_ascii_lowercase() == l)
        .map(|(_, v)| *v)
}

/// The two JRISC index registers (rmac: only R14/R15 can be indexed).
pub const INDEX_REGS: [u8; 2] = [14, 15];
