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
mod error;
mod eval;
mod evaluator;
mod expr;
mod fixerupper;
mod gazm;
mod indexed;
mod item;
mod labels;
mod locate;
mod lsp;
mod macros;
mod messages;
mod node;
mod numbers;
mod opcodes;
mod postfix;
mod register;
mod regutils;
mod scopes;
mod sections;
mod sizer;
mod structs;
mod tokenize;
mod util;
mod token_store;
mod vars;
mod doc;

use anyhow::{Context, Result};
use ctx::Opts;
use lsp::do_lsp;
use messages::{info, messages};

static BANNER: &str = r#"
  __ _  __ _ _____ __ ___
 / _` |/ _` |_  / '_ ` _ \
| (_| | (_| |/ /| | | | | |
 \__, |\__,_/___|_| |_| |_|
 |___/ 6898 Assembler
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli::parse();
    let opts = Opts::from_arg_matches(matches)?;
    let mess = messages::messages();
    mess.set_verbosity(&opts.verbose);

    match opts.build_type {
        ctx::BuildType::LSP => {
            lsp::do_lsp(opts);
            Ok(())
        }

        ctx::BuildType::Build => {
            mess.status(BANNER);

            mess.indent();

            use std::fs;

            let mut ctx = gazm::assemble_from_opts(opts)?;

            for (addr, count) in ctx.asm_out.binary.check_against_referece() {
                println!("{addr:04X} {count}");
            }

            ctx.write_bin_chunks()?;
            ctx.checksum_report();
            ctx.write_lst_file()?;
            ctx.write_sym_file()?;

            mess.deindent();
            mess.info("");

            Ok(())
        }

        ctx::BuildType::Check => panic!("TBD"),
    }
}
