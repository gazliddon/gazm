#![allow(unused_macros)]

use anyhow::{Context, Result};
use glob::glob;
use makehelp::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{env, path::PathBuf};

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

pub fn main() -> Result<()> {
    use helpentry::*;
    let out_dir = env::var("OUT_DIR").context("Trying to get OUTDIR var")?;
    let dest_path = Path::new(&out_dir).join("helptext.rs");

    let path = PathBuf::from("assets/help/*.md");

    let mut paths = vec![];

    for p in glob(&path.to_string_lossy())
        .expect("trying to glob")
        .flatten()
    {
        paths.push(p)
    }

    let all: Result<Vec<HelpEntry>> = paths.iter().map(HelpEntry::new).collect();
    let text = gencode::generate_rust_code(&all?);

    let mut f = File::create(dest_path)?;
    f.write_all(text.as_bytes())?;

    Ok(())
}
