#![forbid(unused_imports)]
/// Utilities for handling registers
use emu6809::cpu::RegEnum;
use std::collections::HashSet;

pub fn reg_to_reg_num(a: RegEnum) -> u8 {
    use RegEnum::*;

    match a {
        D => 0b0000,
        X => 0b0001,
        Y => 0b0010,
        U => 0b0011,
        S => 0b0100,
        PC => 0b0101,
        A => 0b1000,
        B => 0b1001,
        CC => 0b1010,
        DP => 0b1011,
    }
}

pub fn reg_pair_to_flags(source: RegEnum, dest: RegEnum) -> u8 {
    let a = reg_to_reg_num(source);
    let b = reg_to_reg_num(dest);
    (a << 4) | b
}

/// The PSHS/PULS register list byte. `D` aliases the `A` and `B` flag
/// bits, and `U`/`S` share one bit — CPU facts, encoded as a table.
pub fn registers_to_flags(regs: &HashSet<RegEnum>) -> u8 {
    use RegEnum::*;
    let mut registers = 0;
    for (reg, bit) in [
        (CC, 0x01),
        (A, 0x02),
        (B, 0x04),
        (DP, 0x08),
        (X, 0x10),
        (Y, 0x20),
        (U, 0x40),
        (S, 0x40),
        (PC, 0x80),
    ] {
        if regs.contains(&reg) {
            registers |= bit;
        }
    }
    if regs.contains(&D) {
        registers |= 0x02 | 0x04;
    }
    registers
}
