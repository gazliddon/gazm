#[macro_use] extern crate quick_error;
#[allow(dead_code)] mod chunk;
#[allow(dead_code)] mod error;
#[allow(dead_code)] mod rom;
#[allow(dead_code)] mod romloader;

use rom::{Rom};

fn main() {
    let file = "asm/out/all.syms";

    let mut _rom = Rom::from_sym_file(file).unwrap();

    println!("Loc for 0x9900 = {:?}", _rom.get_source_location(0x9900));
    println!("Src for 0x9900 = {:?}", _rom.get_source_line(0x9900));
}