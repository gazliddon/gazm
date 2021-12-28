#![allow(unused_imports)]
#![allow(dead_code)]

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

fn main() {
    use clap::Parser;
    use item::Item::*;

    let ctx = cli::Context::parse();

    let res = tokenize::tokenize(&ctx);

    match res {
        Ok(n) => {
            println!("Succeded!");

            for n in n.tree_iter() {
                let p = n.ctx();

                if p.start == 0 && p.end == 0 {
                    println!("{:?}", n.item())
                }

            }
        },

        Err(e) => {
            println!("{:?}", e)
        }
    }
}


