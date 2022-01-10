#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::collections::VecDeque;
use std::collections::hash_map::DefaultHasher;
use std::env::consts::OS;
use std::hash::Hash;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

mod expr;
mod comments;
mod item;
mod numbers;
mod commands;
mod util;
mod opcodes;
mod register;
mod labels;
mod fileloader;
mod node;
mod ctx;
mod error;
mod locate;
mod symbols;
mod tokenize;
mod cli;
mod assemble;
mod ast;
mod messages;
mod postfix;
mod sourcefile;
mod eval;
mod indexed;

use std::path::PathBuf;
use std::time::Instant;

use ast::ItemWithPos;
use colored::*;
use error::UserError;
use locate::Position;
use romloader::ResultExt;

static BANNER : &str = r#"
  ____                        __    ___   ___   ___
 / ___| __ _ ___ _ __ ___    / /_  ( _ ) / _ \ / _ \
| |  _ / _` / __| '_ ` _ \  | '_ \ / _ \| | | | (_) |
| |_| | (_| \__ \ | | | | | | (_) | (_) | |_| |\__, |
 \____|\__,_|___/_| |_| |_|  \___/ \___/ \___/   /_/
"#;

use crate::item::{ Node, Item };
use crate::messages::Messageize;

pub struct Assembler {
    pub tokens : Node,
    ctx: cli::Context,
    pub ast : ast::Ast,
}

impl Assembler {
    pub fn new(ctx : &cli::Context) -> Result<Self,UserError> {
        let x = messages::messages();

        let msg = format!("Assembling {}", ctx.file.to_string_lossy());
        x.success(&msg);
        x.indent();

        let tokens = tokenize::tokenize(ctx)?;
        let ast = ast::Ast::from_nodes(tokens.clone()).unwrap();

        x.deindent();
        x.success("Complete");

        let ret = Self {
            ast,
            ctx : ctx.clone(),
            tokens
        };

        Ok(ret)
    }

    pub fn get_ast(&self)-> &ast::Ast {
        &self.ast
    }
}
fn print_tree(tree: &ast::AstNodeRef, depth : usize) {

    let dstr = " ".repeat(depth*4);

    println!("{}{:?}",dstr,tree.value().item);

    for n in tree.children() {
        print_tree(&n, depth+1);
    }
    
}

fn main() {

    use clap::Parser;
    use item::Item::*;
    use messages::*;

    let ctx : cli::Context = cli::parse().into();

    let x = messages::messages();

    x.info(BANNER);
    x.intertesting("GASM 6809 Assembler\n");

    x.indent();
    let res = Assembler::new(&ctx);
    x.deindent();
    x.info("");

    match res {
        Ok(asm) => {
            x.success("Succeeded");

            if ctx.dump_ast {
                print_tree(&asm.ast.get_tree().root(), 0);
            }
        },

        Err(e) => {
            let line_num = format!("{}", e.pos.line);
            let spaces = " ".repeat( 1+line_num.len() );
            let bar = format!("{}|", spaces).info();
            let bar_line = format!("{} |", line_num).info();

            println!("{}: {}", "error".error(), e.message.bold());
            println!("   {} {}:{}:{}", "-->".info(),e.file.to_string_lossy(), e.pos.line, e.pos.col);
            println!("{}", bar);
            println!("{} {}", bar_line, e.line);
            println!("{}{}^", bar, " ".repeat(e.pos.col));
        }
    }
}


