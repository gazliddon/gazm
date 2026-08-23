# `gazm` crate notes

This crate contains the production assembler and CLI. Start with `src/main.rs`, `src/cli/`, and `src/assembler/asm.rs` when tracing a command from arguments to output.

- Keep CPU-specific behavior in `cpu6800/` or `cpu6809/`; shared parsing/assembly belongs in the common modules.
- Add regression fixtures under `assets/test_src/` or Tree-sitter corpus files under `gazm-plugin/treesitter-gazm/test/corpus/` when syntax behavior changes.
- Help pages under `assets/help/` are source files; the Rust include is generated at build time.
- Run `cargo test -p gazm` after changes. If a test requires unavailable sibling crates, report that dependency boundary explicitly rather than changing path dependencies.
