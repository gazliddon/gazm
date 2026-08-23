# Gazm

Gazm is a Rust-based assembler for Motorola 6800-family source, currently targeting 6809 and 6800 programs. It provides a reusable assembler library, a command-line interface, generated opcode/help documentation, and an editor plugin with Tree-sitter support.

The project is a Cargo workspace:

| Package | Purpose |
| --- | --- |
| `gazm` | Assembler library and `gazm` CLI |
| `makehelp` | Build-time generator for embedded help text |
| `gazm-plugin` | Neovim Lua integration and Tree-sitter grammar |

## Prerequisites

Use the stable Rust toolchain declared in `rust-toolchain.toml`. The `gazm` crate also expects sibling workspace crates in `../crates` (`emu6800`, `emu6809`, `grl-sources`, `grl-eval`, and `unraveler`). This repository is normally checked out beside those crates.

## Build and test

From the repository root:

```sh
cargo build -p gazm
cargo test -p gazm
cargo test --workspace
```

For a fast syntax/type check, use `cargo check -p gazm`. Formatting is checked with `cargo fmt --all -- --check`.

## CLI quick start

Gazm's config-driven commands read `gazm.toml` (or a path supplied as the first argument):

```sh
cargo run -p gazm -- check path/to/gazm.toml
cargo run -p gazm -- build path/to/gazm.toml
```

The direct `asm` command is useful for one-off files:

```sh
cargo run -p gazm -- asm path/to/program.gazm --mem-size 65536 --max-errors 10
```

Global flags include `-v`/`--verbose`, `--verbose-errors`, and `--no-async`. Other subcommands are `test`, `fmt` (currently a placeholder), and `lsp` (currently a placeholder in the binary).

A minimal config looks like:

```toml
[opts]
project-file = "program.gazm"
cpu = "Cpu6809"
mem-size = 65536
source-mapping = "program.map"
syms-file = "program.sym"
```

Paths are interpreted relative to the config file directory. See `gazm/assets/gazm.toml` for a checked-in example and `gazm/src/opts.rs` for the complete option model.

## Source and output flow

The frontend tokenizes source and recursively discovers `include` files. The semantic layer expands those includes and builds the AST. The assembler resolves scopes/symbols, sizes instructions, compiles bytes, and writes configured outputs such as binaries, maps, symbols, dependency files, and AST/listing artifacts. CPU-specific parsing and instruction encoding live under `gazm/src/cpu6800` and `gazm/src/cpu6809`.

More detail is in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Contributor and AI-agent workflow guidance is in [AGENTS.md](AGENTS.md).
Current priorities, repository boundaries, and Stargate validation steps are in [docs/PROJECT_STATUS.md](docs/PROJECT_STATUS.md).

## Current status

6800 and 6809 assembly paths are present and actively evolving. 6502, 65C02, and Z80 are represented in the CPU enum but are not implemented. The `fmt` and `lsp` CLI branches currently report TODO behavior. Historical release notes and open design work remain in `gazm/README.md`, `todo.md`, and `gazm/tood.md`.
