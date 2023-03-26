#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(try_blocks)]

pub mod as6809;
pub mod asmctx;
pub mod ast;
pub mod astformat;
pub mod async_tokenize;
pub mod binary;
pub mod cli;
pub mod commands;
pub mod comments;
pub mod compile;
pub mod config;
pub mod ctx;
pub mod doc;
pub mod error;
pub mod eval;
pub mod evaluator;
pub mod expr;
pub mod fixerupper;
pub mod fmt;
pub mod gazm;
pub mod indexed;
pub mod item;
pub mod labels;
pub mod locate;
pub mod lookup;
pub mod lsp;
pub mod macros;
pub mod messages;
pub mod newsyms;
pub mod node;
pub mod numbers;
pub mod opcodes;
pub mod parse;
pub mod register;
pub mod regutils;
pub mod scopes;
pub mod sections;
pub mod sizer;
pub mod structs;
pub mod token_store;
pub mod tokenize;
pub mod vars;
