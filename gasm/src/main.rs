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

    if let Ok(n) = res {
        for i in n.tree_iter().filter(|x| x.item() == &Assignment) {
            println!("{:?}",i)
        }
    }
}


