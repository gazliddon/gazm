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

use crate::ctx::{CheckSum, Context, Opts};
use crate::error::{GResult, GazmError};
use ::gazm::{gazm::with_state, messages::info};
use emu::utils::sources::fileloader::FileIo;
use emu::utils::sources::SourceDatabase;
use lsp::do_lsp;
use messages::messages;
use std::path::PathBuf;

use sha1::{Digest, Sha1};

static BANNER: &str = r#"
  __ _  __ _ _____ __ ___
 / _` |/ _` |_  / '_ ` _ \
| (_| | (_| |/ /| | | | | |
 \__, |\__,_/___|_| |_| |_|
 |___/
"#;

use crate::error::*;

use anyhow::Result;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use anyhow::Context;

    let matches = cli::parse();
    let opts = Opts::from_arg_matches(matches)?;

    if opts.build_type == ctx::BuildType::LSP {
        lsp::do_lsp(opts.clone());
        return Ok(());
    }

    use crate::messages::Verbosity;
    let x = messages::messages();

    x.set_verbosity(&opts.verbose);

    // if ctx.opts.verbose != Verbosity::Silent {
    //     let m = format!("Verboisty: {:?}", ctx.opts.verbose);
    //     x.status(m)
    // };

    status_mess!("{}", BANNER);
    status_mess!("GASM 6809 Assembler\n");

    x.indent();

    use std::fs;

    let ctx_shared = gazm::assemble(opts)?;

    with_state(&ctx_shared, |ctx| -> GResult<()> {
        for (addr, count) in ctx.binary.check_against_referece() {
            println!("{addr:04X} {count}");
        }

        ctx.write_bin_chunks()?;
        ctx.checksum_report();
        ctx.write_lst_file()?;

        if let Some(sym_file) = &ctx.opts.syms_file {
            let _syms = ctx.symbols.clone();

            // let sd : SourceDatabase = ctx.into();

            status_mess!("Writing symbols: {}", sym_file);

            // sd.write_json(sym_file).with_context(||format!("Unable to write {sym_file}"))?;

            if let Some(deps) = &ctx.opts.deps_file {
                status_mess!("Writing deps file : {deps}");

                let as_string = |s: &PathBuf| -> String { s.to_string_lossy().into() };

                let read: Vec<String> = ctx
                    .get_source_file_loader()
                    .get_files_read()
                    .iter()
                    .map(as_string)
                    .collect();
                let written: Vec<String> = ctx
                    .get_source_file_loader()
                    .get_files_written()
                    .iter()
                    .map(as_string)
                    .collect();

                let deps_line_2 = format!("{} : {sym_file}", written.join(" \\\n"));
                let deps_line = format!("{deps_line_2}\n{sym_file} : {}", read.join(" \\\n"));

                fs::write(deps, deps_line).with_context(|| format!("Unable to write {deps}"))?;
            }
        }
        Ok(())
    })?;

    x.deindent();
    x.info("");

    Ok(())
}
