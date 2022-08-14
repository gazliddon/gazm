#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(try_blocks)]

pub mod as6809;
pub mod ast;
pub mod astformat;
pub mod binary;
pub mod cli;
pub mod commands;
pub mod comments;
pub mod ctx;
pub mod error;
pub mod eval;
pub mod expr;
pub mod indexed;
pub mod item;
pub mod labels;
pub mod locate;
pub mod macros;
pub mod messages;
pub mod node;
pub mod numbers;
pub mod opcodes;
pub mod postfix;
pub mod register;
pub mod scopes;
pub mod sections;
pub mod structs;
pub mod tokenize;
pub mod util;
pub mod sizer;
pub mod compile;
pub mod gazm;
pub mod evaluator;
pub mod asmctx;
pub mod fixerupper;
pub mod regutils;

