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
cargo build -p gazm
make -C ../stargate clean
make -C ../stargate ASM="$PWD/target/debug/gazm build"
```

The final Stargate command must leave all generated ROMs byte-identical to
`../stargate/orig/roms`.

## Known architectural risk

The assembler is partway through a CPU abstraction refactor. The shared
frontend and assembler pipeline should own language-independent behavior;
each CPU backend should own instruction parsing, sizing, encoding, and CPU
state such as direct-page handling. Avoid adding more enum-based dispatch or
temporary `todo!()` bridges without a focused test and an explicit reason.
