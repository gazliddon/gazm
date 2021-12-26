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


// use item::{Item, Node};

// use nom::branch::alt;
// use nom::bytes::complete::tag_no_case;

// use nom::character::complete::{
//     line_ending, multispace0, multispace1
// };
// use nom::character::{is_alphabetic, is_space};
// use nom::combinator::{eof, opt, all_consuming};
// use nom::sequence::{pair, tuple};
// use nom::IResult;

// use opcodes::{parse_opcode, opcode_token};
use std::collections::HashMap;
use std::fs;

use std::hash::Hash;
use std::os::unix::prelude::JoinHandleExt;
use std::path::{Path, PathBuf,};
use std::rc::Rc;

use clap::Parser;
#[derive(Parser, Debug)]
#[clap(about, version, author)]

pub struct Context {
#[clap(long)]
    verbose: bool,
    #[clap(short, long)]
    file : PathBuf,
    #[clap(short, long)]
    out: Option<String>
}

use std::env;
use std::io;

fn main() {

    let ctx = Context::parse();

    let res = tokenize::tokenize(&ctx);

    if let Ok(n) = res {

        for i in n.tree_iter().filter(|x| x.item() == &item::Item::Assignment) {
            println!("{:?}",i)
        }
    }
}


