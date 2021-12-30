#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::hash_map::DefaultHasher;
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

use std::path::PathBuf;

use colored::*;
use error::UserError;
use romloader::ResultExt;

static BANNER : &str = r#"
  ____                        __    ___   ___   ___
 / ___| __ _ ___ _ __ ___    / /_  ( _ ) / _ \ / _ \
| |  _ / _` / __| '_ ` _ \  | '_ \ / _ \| | | | (_) |
| |_| | (_| \__ \ | | | | | | (_) | (_) | |_| |\__, |
 \____|\__,_|___/_| |_| |_|  \___/ \___/ \___/   /_/
"#;

fn doit(ctx : &cli::Context) -> Result<(),UserError> {
    
    let tokens = tokenize::tokenize(&ctx)?;
    println!();
    let _bin = assemble::assemble(&ctx, tokens)?;

    Ok(())
}

fn main() {
    use clap::Parser;
    use item::Item::*;

    let ctx : cli::Context = cli::parse().into();
    println!("{}", BANNER.bold().blue());
    println!("{}", "GASM 6809 Assembler".purple().bold());
    println!();

    let res = doit(&ctx);

    match res {
        Ok(n) => {
            let msg = "\nSucceded!".green().bold();
            println!("{}", msg);

            if ctx.pretty_dump_ast {
                println!("{:#?}", n);
            } else {
                if ctx.dump_ast {
                    println!("{:?}", n);
                }
            }
        },

        Err(e) => {
            let line_num = format!("{}", e.pos.line);
            let spaces = " ".repeat( 1+line_num.len() );
            let bar = format!("{}|", spaces).blue().bold();
            let bar_line = format!("{} |", line_num).blue().bold();

            println!("{}: {}", "error".red().bold(), e.message.bold());
            println!("   {} {}:{}:{}", "-->".blue(),e.file.to_string_lossy(), e.pos.line, e.pos.col);
            println!("{}", bar);
            println!("{} {}", bar_line, e.line);
            println!("{}{}^", bar, " ".repeat(e.pos.col));
        }
    }
}


