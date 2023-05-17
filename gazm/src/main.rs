#![allow(unused_imports)]
#![allow(dead_code)]
// #![deny(clippy::pedantic)]
mod parse6809;
mod item6809;
mod asmctx;
mod ast;
mod astformat;
mod async_tokenize;
mod binary;
mod cli;
mod compile;
mod config;
mod ctx;
mod error;
mod eval;
mod evaluator;
mod fixerupper;
mod fmt;
mod gazm;
mod item;
mod lookup;
mod lsp;
mod messages;
mod newsyms;
mod node;
mod parse;
mod regutils;
mod scopes;
mod sections;
mod sizer;
mod token_store;
mod tokenize;
mod vars;
mod opts;

use anyhow::{Context, Result};
use emu::utils::sources::SourceDatabase;
use ::gazm::opts::BinReference;
use opts::{ Opts, BuildType };
use error::GResult;
use messages::{info, messages};

static BANNER: &str = r#"
  __ _  __ _ _____ __ ___
 / _` |/ _` |_  / '_ ` _ \
| (_| | (_| |/ /| | | | | |
 \__, |\__,_/___|_| |_| |_|
 |___/ 6898 Assembler
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env::{current_dir, set_current_dir};

    let matches = cli::parse();

    let opts = Opts::from_arg_matches(matches)?;

    let mess = messages::messages();
    mess.set_verbosity(&opts.verbose);

    // Todo move directory handling into assemble_from_opts
    // probably as a function of Opts
    let cur_dir = current_dir().unwrap();

    if let Some(assemble_dir) = &opts.assemble_dir {
        std::env::set_current_dir(assemble_dir)?;
    }

    let mut asm = gazm::Assembler::new(opts.clone());

    match opts.build_type {
        BuildType::Format => {
            status_mess!("Format file");
            fmt::fmt(&opts)?;
        }

        BuildType::Lsp => {
            status_mess!("LSP");
            lsp::do_lsp(opts)?;
        }

        // Build of check to see if build is okay
        BuildType::Build | BuildType::Check => {
            status_mess!("{BANNER}");
            mess.indent();
            status_mess!("Verbosity: {:?}", &opts.verbose);

            if opts.no_async {
                status_mess!("Async: NO ASYNC");
            }

            asm.assemble()?;

            // Only write outputs if this is of buildtype Build
            if opts.build_type == BuildType::Build {
                asm.write_outputs()?;
            }

            mess.deindent();
            info_mess!("")
        }
    };

    set_current_dir(cur_dir)?;

    Ok(())
}
