use std::path::{Path, PathBuf,};
use clap::Parser;
#[derive(Parser, Debug)]
#[clap(about, version, author)]

pub struct Context {
#[clap(long)]
    pub verbose: bool,
    #[clap(short, long)]
    pub file : PathBuf,
    #[clap(short, long)]
    pub out: Option<String>
}
