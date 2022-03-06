#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(try_blocks)]
#![feature(backtrace)]
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env::consts::OS;
use std::hash::Hash;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

mod as6809;
mod assemble;
mod ast;
mod astformat;
mod binary;
mod cli;
mod commands;
mod comments;
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
mod structs;
// mod sourcefile;
// mod symbols;
mod sections;
mod tokenize;
mod util;

use std::path::PathBuf;
use std::process::abort;
use std::time::Instant;

use ast::ItemWithPos;
use colored::*;
use error::UserError;
use messages::{debug, info, status};
use romloader::sources::FileIo;
use romloader::ResultExt;

static BANNER: &str = r#"
  ____                        __    ___   ___   ___
 / ___| __ _ ___ _ __ ___    / /_  ( _ ) / _ \ / _ \
| |  _ / _` / __| '_ ` _ \  | '_ \ / _ \| | | | (_) |
| |_| | (_| \__ \ | | | | | | (_) | (_) | |_| |\__, |
 \____|\__,_|___/_| |_| |_|  \___/ \___/ \___/   /_/
"#;

use crate::ast::AstNodeRef;
use crate::cli::WriteBin;
use crate::error::*;
use crate::item::{Item, Node};
use crate::messages::Messageize;

use assemble::Assembler;

fn assemble(ctx: &mut cli::Context) -> Result<assemble::Assembled, Box<dyn std::error::Error>> {
    use assemble::Assembler;
    use ast::Ast;

    let tokens = tokenize::tokenize(ctx)?;

    let ast = Ast::from_nodes(tokens, ctx)?;

    let mut asm = Assembler::new(ast)?;

    asm.size()?;

    let ret = asm.assemble()?;

    Ok(ret)
}

fn print_tree(tree: &ast::AstNodeRef, depth: usize) {
    let dstr = " ".repeat(depth * 4);

    println!("{}{:?}", dstr, tree.value().item);

    for n in tree.children() {
        print_tree(&n, depth + 1);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use anyhow::Context;

    use clap::Parser;
    use item::Item;
    use messages::*;

    let mut ctx: cli::Context = cli::parse().into();

    let x = messages::messages();
    x.set_verbosity(&ctx.verbose);

    // x.status(BANNER);
    // x.status("GASM 6809 Assembler\n");

    x.indent();

    let ret = assemble(&mut ctx)?;

    use std::fs;

    if let Some(sym_file) = &ctx.syms_file {
        x.status(format!("Writing symbols: {}", sym_file));
        let j = serde_json::to_string_pretty(&ret.database).expect("Unable to serialize to json");
        fs::write(sym_file, j).with_context(|| format!("Unable to write {sym_file}"))?;

        if let Some(deps) = &ctx.deps_file {
            x.status(format!("Writing deps file : {deps}"));

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

            fs::write(deps,deps_line).with_context(|| format!("Unable to write {deps}"))?;
        }
    }

    x.deindent();

    x.info("");

    Ok(())
}
