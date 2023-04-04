#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(try_blocks)]

mod as6809;
mod asmctx;
mod ast;
mod astformat;
mod async_tokenize;
mod binary;
mod cli;
mod commands;
mod comments;
mod compile;
mod config;
mod ctx;
mod doc;
mod error;
mod eval;
mod evaluator;
mod expr;
mod fixerupper;
mod fmt;
mod gazm;
mod indexed;
mod item;
mod labels;
mod locate;
mod lookup;
mod lsp;
mod macros;
mod messages;
mod newsyms;
mod node;
mod numbers;
mod opcodes;
mod parse;
mod register;
mod regutils;
mod scopes;
mod sections;
mod sizer;
mod structs;
mod token_store;
mod tokenize;
mod vars;

use anyhow::{Context, Result};
use ctx::Opts;
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
    mess.status(format!("Verbosity {:?}", &opts.verbose));

    // Todo move directory handling into assemble_from_opts
    // probably as a function of Opts
    let cur_dir = current_dir().unwrap();

    if let Some(assemble_dir) = &opts.assemble_dir {
        std::env::set_current_dir(assemble_dir)?;
    }

    let asm = gazm::Assembler::new(opts.clone());

    match opts.build_type {
        ctx::BuildType::Format => {
            mess.status("Format file");
            fmt::fmt(&opts)
        }

        ctx::BuildType::LSP => {
            mess.status("LSP");
            lsp::do_lsp(opts)?;
        }

        // Build of check to see if build is okay
        ctx::BuildType::Build | ctx::BuildType::Check => {
            mess.status(BANNER);
            mess.indent();
            asm.assemble()?;

            // Only write outputs if this is of buildtype Build
            if opts.build_type == ctx::BuildType::Build {
                asm.write_outputs()?;
            }

            mess.deindent();
            mess.info("");
        }
    };

    set_current_dir(cur_dir)?;

    Ok(())
}
