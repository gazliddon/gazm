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

## Open work: metadata contract (read this before touching artifacts)

`docs/METADATA_CONTRACT.md` is the contract for the `.map`/`.sym` artifacts
and the planned `gazm-metadata` reader library. The williams-emu debugger
is the consumer. If you touch `encode_artifact`, `ARTIFACT_VERSION`,
`source-mapping`/`syms-file` options, or anything that changes the on-disk
format, follow that contract and update it. In particular:

- The `[sections]` TOML block in project configs is dead config — remove
  it and `SectionToml`; real sections come from the in-source `section`
  directive and should be persisted (see contract §6).
- The per-target `source-mapping`/`syms-file` options are being replaced
  by a single `metadata = true|false` switch (contract §5).
- The envelope gains an optional `TargetInfo` header at v4 (contract §4);
  v3 files must keep loading.

## Work boundaries (active project)

There are two agents working in this repo in parallel. Respect the split:

- `gazm-metadata/` — the **reader library** for the `.map`/`.sym`
  artifacts (owned by the williams-emu debugger agent). Do NOT edit it;
  you may review and comment, but code changes there are theirs.
- `gazm/src/**` + config — the **writer side** (owned by the gazm agent):
  `TargetInfo` header at v4, the single `metadata` switch, in-asm section
  persistence, removal of dead `[sections]`/`SectionToml`.
- `docs/METADATA_CONTRACT.md` — shared format truth. Any format change
  updates it first; both sides implement against it.
- Only shared file either agent may edit: the workspace root `Cargo.toml`
  (member list) — keep changes additive.
