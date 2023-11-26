use makehelp::{gencode, helpentry::HelpEntry};

use anyhow::{Context, Result};
use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "gs")]
#[structopt(version = "0.1.0")]
#[structopt(about = "git status checker")]
#[structopt(author = "gazaxian")]
#[structopt(rename_all = "kebab-case")]
pub struct Opts {
    #[structopt(short, long,)]
    verbose: bool,
    #[structopt(short, long, parse(from_os_str))]
    out_file: Option<PathBuf>,
    #[structopt(name = "FILE", parse(from_os_str))]
    paths: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let opts = Opts::from_args();

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
