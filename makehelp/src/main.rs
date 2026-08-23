use makehelp::{gencode, helpentry::HelpEntry};

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

/// Help generator for the Gazm assembler
#[derive(Parser, Debug)]
#[command(name = "makehelp", version, author = "gazaxian")]
pub struct Opts {
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long)]
    out_file: Option<PathBuf>,
    #[arg(name = "FILE")]
    paths: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let all: Result<Vec<HelpEntry>> = opts.paths.iter().map(HelpEntry::new).collect();

    let all = all.context("Loading help files")?;

    let text = gencode::generate_rust_code(&all);

    if opts.verbose {
        println!("{text}");
    }

    if let Some(out_file) = opts.out_file {
        println!("Now write {out_file:?}");
    }

    Ok(())
}
