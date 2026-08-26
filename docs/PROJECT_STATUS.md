# Project status and current goals

## What Gazm has already achieved

Gazm is a Rust assembler that can assemble the Stargate 6809 source and produce
binary-accurate ROM images. The Stargate build is the primary compatibility
fixture: after building Gazm from source, assemble Stargate and compare every
generated ROM with `../stargate/orig/roms`.

The project also contains emulator-related code and sibling CPU crates. The
current focus is the assembler and emulators; the plugin, formatter, LSP, and
other auxiliary tools are not priorities right now.

## Current goals

1. Understand and improve the quality of the existing assembler and emulator
   code.
2. Finish the multi-CPU assembler design while keeping the existing assembly
   directives and source language stable.
3. Make CPU-specific parsing, sizing, and encoding explicit and testable,
   without scattering CPU conditionals through the shared assembler pipeline.
4. Preserve 6809/Stargate binary compatibility while introducing the new
   abstraction.
5. Build a reliable regression suite around CPU selection, directives,
   instruction sizing, emitted bytes, and Stargate ROM comparisons.
6. Make the codebase easier for AI agents and human contributors to navigate.

## Open technical debt

Tracked items that are not urgent but should be scheduled:

- **Migrate bincode 1.x -> 2.x (or postcard)** for symbol/source-map
  artifacts. `bincode` 1.3 is flagged unmaintained by
  [RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141);
  it is the only runtime dependency with an audit warning. Usage is
  confined to a single `bincode::serialize` call in
  `gazm/src/assembler/writers.rs` (`encode_artifact`), behind the
  versioned header documented in `docs/ARTIFACT_FORMAT.md`. Migrating to
  bincode 2.x changes the on-disk payload encoding, so bump
  `ARTIFACT_VERSION` (currently `3`) and confirm consumers reject the old
  version cleanly. `paste` (unmaintained, RUSTSEC-2024-0436) is a
  transitive compile-time dependency of `unraveler` in the shared crates
  workspace; address it there if it ever matters.

## Repository and crate structure

Gazm and the reusable support crates are currently separate Cargo workspaces:

```text
development/gazm/      assembler product and CLI
development/crates/    shared CPU, source, symbol, evaluation, and parser crates
development/stargate/  binary-accuracy integration fixture
```

This separation is intentional for now. The sibling crates are used by more
than Gazm (including other local projects), so they should not be moved into
the Gazm repository until their ownership and API boundaries are clearer.

### Current guidance

- Keep reusable CPU/emulator and assembler-infrastructure code in
  `development/crates`.
- Keep Gazm-specific orchestration, CLI behavior, and product decisions in
  `development/gazm`.
- Treat changes in `development/crates` as changes to a shared library surface;
  check its workspace and downstream users before reorganizing APIs.
- The Lisp/compiler project `development/clad` has its own workspace and a
  local `unraveler` fork; do not assume it uses the same parser crate as Gazm.

## Validation commands

From `development/gazm`:

```sh
cargo check --workspace
cargo test -p gazm
cargo clippy -p gazm --all-targets
cargo build -p gazm
make -C ../stargate clean
make -C ../stargate ASM="$PWD/target/debug/gazm build"
```

`cargo clippy` must pass clean: the crate denies warnings (`#![deny(warnings)]`
in `lib.rs`/`main.rs`), with `clippy::result_large_err` deliberately allowed —
`GResult<T>` returns the rich `GazmErrorKind` by value (largest variant well
over clippy's 128-byte threshold) because it carries user-facing diagnostic
payloads. Prefer boxing the kind only if that becomes a measured hot path.

The final Stargate command must leave all generated ROMs byte-identical to
`../stargate/orig/roms`.

## Known architectural risk

The assembler is partway through a CPU abstraction refactor. The shared
frontend and assembler pipeline should own language-independent behavior;
each CPU backend should own instruction parsing, sizing, encoding, and CPU
state such as direct-page handling. Avoid adding more enum-based dispatch or
temporary `todo!()` bridges without a focused test and an explicit reason.
