#![deny(warnings)]
#![allow(unused_imports)]
#![allow(dead_code)]
pub mod assembler;
pub mod cli;
pub mod fmt;
pub mod frontend;
pub mod lsp;
pub mod messages;
pub mod opts;
pub mod error;
pub mod cpu6809;

mod astformat;
mod docs;
mod gazmsymbols;
mod lookup;
mod sections;
mod semantic;
mod utils;
mod vars;
mod help;
