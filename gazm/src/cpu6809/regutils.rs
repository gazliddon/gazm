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

pub fn registers_to_flags(regs: &HashSet<RegEnum>) -> u8 {
    use RegEnum::*;
    let mut registers = 0;

    if regs.contains(&CC) {
        registers |= 0x01;
    }

    if regs.contains(&A) {
        registers |= 0x02;
    }
    if regs.contains(&B) {
        registers |= 0x04;
    }

    if regs.contains(&DP) {
        registers |= 0x08;
    }

    if regs.contains(&X) {
        registers |= 0x10;
    }

    if regs.contains(&Y) {
        registers |= 0x20;
    }

    if regs.contains(&D) {
        registers |= 0x02;
        registers |= 0x04;
    }

    if regs.contains(&U) || regs.contains(&S) {
        registers |= 0x40;
    }

    if regs.contains(&PC) {
        registers |= 0x80;
    }
    registers
}
