#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env::consts::OS;
use std::hash::Hash;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

mod assemble;
mod ast;
mod astformat;
mod cli;
mod commands;
mod comments;
mod ctx;
mod error;
mod eval;
mod expr;
mod fileloader;
mod indexed;
mod item;
mod labels;
mod locate;
mod messages;
mod node;
mod numbers;
mod opcodes;
mod postfix;
mod register;
mod scopes;
mod sourcefile;
mod symbols;
mod tokenize;
mod util;

use std::path::PathBuf;
use std::time::Instant;

use ast::ItemWithPos;
use colored::*;
use error::UserError;
use locate::Position;
use romloader::ResultExt;
use util::{debug, info};

static BANNER: &str = r#"
  ____                        __    ___   ___   ___
 / ___| __ _ ___ _ __ ___    / /_  ( _ ) / _ \ / _ \
| |  _ / _` / __| '_ ` _ \  | '_ \ / _ \| | | | (_) |
| |_| | (_| \__ \ | | | | | | (_) | (_) | |_| |\__, |
 \____|\__,_|___/_| |_| |_|  \___/ \___/ \___/   /_/
"#;

use crate::item::{Item, Node};
use crate::messages::Messageize;
use crate::ast::AstNodeRef;
use crate::error::*;


use assemble::Assembler;

fn assemble(ctx: &cli::Context) -> Result<(), Box<dyn std::error::Error>> {
    let msg = format!("Assembling {}", ctx.file.to_string_lossy());

    info(&msg, |x| {
        use assemble::Assembler;
        use ast::Ast;

        let tokens = tokenize::tokenize(ctx)?;

        let ast = Ast::from_nodes(tokens.clone())?;

        let mut asm : Assembler = ast.into();

        asm.size()?;
        asm.assemble()?;

        x.success("Complete");

        Ok(())
    })
}



fn print_tree(tree: &ast::AstNodeRef, depth: usize) {
    let dstr = " ".repeat(depth * 4);

    println!("{}{:?}", dstr, tree.value().item);

    for n in tree.children() {
        print_tree(&n, depth + 1);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    use item::Item::*;
    use messages::*;

    let ctx: cli::Context = cli::parse().into();

    let x = messages::messages();

    x.info(BANNER);
    x.intertesting("GASM 6809 Assembler\n");

    x.indent();

    assemble(&ctx)?;

    x.deindent();
    x.info("");

    Ok(())
}
