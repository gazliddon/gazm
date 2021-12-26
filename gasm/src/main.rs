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

fn main() {
    use clap::Parser;

    let ctx = cli::Context::parse();

    let res = tokenize::tokenize(&ctx);

    if let Ok(n) = res {
        for i in n.tree_iter().filter(|x| x.item() == &item::Item::Assignment) {
            println!("{:?}",i)
        }
    }
}


