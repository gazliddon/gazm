#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(try_blocks)]

mod as6809;
mod ast;
mod astformat;
mod binary;
mod cli;
mod commands;
mod comments;
mod ctx;
mod error;
mod eval;
mod expr;
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
mod scopes;
mod sections;
mod structs;
mod tokenize;
mod async_tokenize;
mod util;
mod sizer;
mod compile;
mod gazm;
mod evaluator;
mod asmctx;
mod fixerupper;
mod regutils;
mod config;

use std::path::PathBuf;
use crate::error::{GazmError, GResult };
use crate::gazm::Gazm;

use crate::ctx::Context;
use emu::utils::sources::{FileIo, SourceDatabase};

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
    
    let mut a = Gazm::new(ctx_shared.clone());
    a.assemble_file(&project_file)?;

    let ctx = ctx_shared.lock().unwrap();

    for (addr,count) in ctx.binary.check_against_referece() {
        println!("{addr:04X} {count}");
    }

    if let Some(lst_file) = &ctx.opts.lst_file {
        let text = ctx.lst_file.lines.join("\n");
        fs::write(lst_file, text).with_context(|| format!("Unable to write list file {lst_file}"))?;
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

    if let Some(_sym_file) = &ctx.opts.syms_file {

//         let _syms = ctx.symbols.clone();

//         let sd : SourceDatabase = ( &ctx ).into();

//         status_mess!("Writing symbols: {}", sym_file);

//         sd.write_json(sym_file).with_context(||format!("Unable to write {sym_file}"))?;

//         if let Some(deps) = &opts.deps_file {
//             status_mess!("Writing deps file : {deps}");

//             let as_string = |s: &PathBuf| -> String { s.to_string_lossy().into() };

//             let read: Vec<String> = ctx
//                 .get_source_file_loader()
//                 .get_files_read()
//                 .iter()
//                 .map(as_string)
//                 .collect();
//             let written: Vec<String> = ctx
//                 .get_source_file_loader()
//                 .get_files_written()
//                 .iter()
//                 .map(as_string)
//                 .collect();

//             let deps_line_2 = format!("{} : {sym_file}", written.join(" \\\n"));
//             let deps_line = format!("{deps_line_2}\n{sym_file} : {}", read.join(" \\\n"));

//             fs::write(deps, deps_line).with_context(|| format!("Unable to write {deps}"))?;
//         }
    }

    x.deindent();
    x.info("");

    Ok(())
}
