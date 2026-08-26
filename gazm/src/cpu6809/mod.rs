#![deny(unused_imports)]
pub mod assembler;
pub mod frontend;
mod regutils;

/// The shared AST node kind, aliased for the backend's `From` impls.
pub type NodeKind = crate::frontend::AstNodeKind;
