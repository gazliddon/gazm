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

use crate::ctx::Context;
use crate::error::{GResult, GazmError};
use ::gazm::messages::messages;
use ::gazm::{ctx::CheckSum, gazm::with_state, messages::info};
use emu::utils::sources::{FileIo, SourceDatabase};
use std::path::PathBuf;

use sha1::{Digest, Sha1};

static BANNER: &str = r#"
  ____                        __    ___   ___   ___
 / ___| __ _ ___ _ __ ___    / /_  ( _ ) / _ \ / _ \
| |  _ / _` / __| '_ ` _ \  | '_ \ / _ \| | | | (_) |
| |_| | (_| \__ \ | | | | | | (_) | (_) | |_| |\__, |
 \____|\__,_|___/_| |_| |_|  \___/ \___/ \___/   /_/
"#;

use crate::error::*;

use anyhow::Result;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use anyhow::Context;

    let x_opts: ctx::Opts = cli::parse().into();
    let ctx: ctx::Context = x_opts.clone().into();

    use crate::messages::Verbosity;

    let x = messages::messages();

    x.set_verbosity(&ctx.opts.verbose);

    if ctx.opts.verbose != Verbosity::Silent {
        let m = format!("Verboisty: {:?}", ctx.opts.verbose);
        x.status(m)
    };

    // x.status(BANNER);
    status_mess!("GASM 6809 Assembler\n");

    x.indent();

    use std::fs;

    use std::sync::{Arc, Mutex};

    let project_file = ctx.opts.project_file.clone();
    let ctx_shared = Arc::new(Mutex::new(ctx));

    gazm::assemble_file(&ctx_shared, &project_file)?;

    with_state(&ctx_shared, |ctx| -> GResult<()> {
        for (addr, count) in ctx.binary.check_against_referece() {
            println!("{addr:04X} {count}");
        }

        // Check any checksums

        for (name, csum) in &ctx.opts.checksums {
            let mut hasher = Sha1::new();
            let data = ctx.binary.get_bytes(csum.addr, csum.size);
            hasher.update(data);
            let this_hash = hasher.digest().to_string().to_lowercase();
            let expected_hash = csum.sha1.to_lowercase();

            let mess = messages::messages();
            let old_verb = mess.get_verbosity();
            mess.set_verbosity(&Verbosity::Info);

            if this_hash != expected_hash {
                status_mess!("{name} : ❌")
            } else {
                status_mess!("{name} : ✅")
            }

            mess.set_verbosity(&old_verb);
        }

        if let Some(lst_file) = &ctx.opts.lst_file {
            let text = ctx.lst_file.lines.join("\n");
            fs::write(lst_file, text)
                .with_context(|| format!("Unable to write list file {lst_file}"))?;
            status_mess!("Written lst file {lst_file}");
        }

        if let Some(ast_file) = &ctx.opts.ast_file {
            status_mess!("Writing ast: {}", ast_file.to_string_lossy());
            status_err!("Not done!");
            if let Some(ast) = &ctx.ast {
                let x = astformat::as_string(ast.root());
                println!("{x}");
            } else {
                status_err!("No AST file to write");
            }
        }

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
