//! Reader library for gazm's versioned `.map`/`.sym` metadata artifacts.
//!
//! The Stargate emulator's debugger consumes these to annotate
//! disassembly with source lines, resolve symbol names, and map
//! addresses to `file:line`.  See `docs/METADATA_CONTRACT.md` in the
//! gazm workspace for the on-disk format contract.
//!
//! This crate deliberately depends only on `bincode`/`serde` and the
//! grl crates — not on the `gazm` toolchain — so the emulator can link
//! it without pulling in the CLI/assembler stack.

pub mod envelope;
pub mod sourcemap;
pub mod symbols;
pub mod target;

pub use envelope::{decode_artifact, Artifact, Magic};
pub use sourcemap::{InstructionBoundary, SourceLocation, SourceMap};
pub use symbols::Symbols;
pub use target::Target;
