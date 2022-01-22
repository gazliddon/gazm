#![allow(dead_code)]

#[macro_use]
extern crate quick_error;

mod chunk;
mod error;
mod location;
mod rom;
mod romloader;
mod sourcestore;

use rom::Rom;

fn main() {
    let file = "asm/out/test.syms";

    let mut _rom = Rom::from_sym_file(file).unwrap();
}
