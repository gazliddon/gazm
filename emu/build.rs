use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;


extern crate romloader;

fn main() {
    let dbase = romloader::Dbase::from_filename("src/cpu/resources/opcodes.json");
    let source = format!("{}", dbase);
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("isa_macros.rs");
    let mut f = File::create(&dest_path).unwrap();
    f.write_all(source.as_bytes()).unwrap();
}

