#![deny(warnings)]
// The repository currently contains legacy parser/LSP paths that are compiled
// but not all wired into the active CLI. Keep these allowances local to the
// crate until those paths are either removed or brought back into use.
#![allow(unused_imports)]
#![allow(dead_code)]
pub mod assembler;
pub mod cli;
pub mod cpu6800;
pub mod cpu6809;
pub mod cpukind;
pub mod error;
pub mod fmt;
pub mod frontend;
pub mod lsp;
pub mod messages;
pub mod opts;

mod astformat;
mod docs;
mod gazmsymbols;
mod help;
mod lookup;
mod sections;
mod semantic;
mod vars;
