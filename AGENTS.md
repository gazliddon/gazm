# Working with this repository

This repository is a Rust workspace for **Gazm**, a Motorola 6800/6809 assembly toolchain, plus its editor integration.

## Orientation

- `gazm/` — the assembler library and `gazm` command-line binary.
- `gazm/src/frontend/` — target-independent tokenization and parsing.
- `gazm/src/cpu6800/` and `gazm/src/cpu6809/` — CPU-specific lexers, parsers, instruction sets, and assemblers.
- `gazm/src/semantic/` — include expansion and semantic AST processing.
- `gazm/src/assembler/` — symbol resolution, sizing, compilation, binary generation, and output writers.
- `gazm/src/cli/` — command-line parsing and `gazm.toml` loading.
- `gazm/assets/` — help text and assembly fixtures used by the build and tests.
- `gazm-plugin/` — Neovim Lua integration and the Tree-sitter grammar/package.
- `makehelp/` — build-time helper that turns `gazm/assets/help/*.md` into generated Rust.

Read `README.md` for user-facing usage and `docs/ARCHITECTURE.md` for the data flow.

## Safe workflow for agents

1. Inspect `git status --short` before editing. Existing changes belong to the user; do not reset, discard, or reformat them.
2. Keep changes scoped to the requested behavior. Prefer `apply_patch` for hand-authored files.
3. Use the workspace root for Cargo commands. The `gazm` crate has path dependencies on sibling crates under `../crates`, so a standalone checkout of only this directory cannot build.
4. Run `cargo fmt --all -- --check` and targeted tests after Rust changes. Run `cargo check -p gazm` when dependencies are available.
5. Treat `gazm/assets/help/helptext.rs` as generated output in `OUT_DIR`; edit the Markdown source under `gazm/assets/help/` instead.
6. Do not commit build products (`target/`) or editor-local files.

## Design constraints

- Preserve support for both 6800 and 6809 paths; do not silently route one CPU through the other.
- User-facing diagnostics are part of the CLI contract. Prefer the existing error types and collectors over ad-hoc `unwrap`/`println!` paths.
- Configuration is TOML and is resolved relative to the config file's directory. Keep examples valid TOML.
- The LSP command and several future CPU backends are currently incomplete; document limitations instead of presenting them as working features.
