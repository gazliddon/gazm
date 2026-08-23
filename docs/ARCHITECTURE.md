# Architecture

## Runtime pipeline

```text
CLI / gazm.toml
        |
        v
    Opts + CPU selection
        |
        v
  Tokenizer + include discovery  ---> source positions / diagnostics
        |
        v
  Parser (shared frontend + CPU frontend)
        |
        v
  Semantic AST + include expansion
        |
        v
  Assembler (scopes, symbols, sizing, fixups)
        |
        v
  Binary / map / symbols / deps / AST / listing writers
```

`gazm/src/main.rs` owns process-level command dispatch. `gazm/src/cli/parse.rs` maps Clap subcommands to `Opts`; `gazm/src/cli/config.rs` deserializes TOML and makes the config directory the assembly working directory.

## Important modules

- `frontend/`: common tokens, source locations, include tracking, and parser infrastructure.
- `cpu6800/`, `cpu6809/`: target-specific token and opcode parsing plus encoding/sizing rules.
- `semantic/`: AST context and recursive include inlining. This is where a source tree becomes a semantically assembled unit.
- `assembler/`: the central `Assembler` type and its compilation pipeline. `binary.rs` models memory and output ranges; `scopes.rs` and `scopetracker.rs` handle symbol scope; `sizer.rs`, `compile.rs`, and `fixerupper.rs` handle instruction layout and deferred references; `writers.rs` emits outputs.
- `error.rs`: shared user-error and diagnostic collection. Preserve this path when adding diagnostics so CLI output remains consistent.
- `lsp/`: server/backend types exist, but the executable's `lsp` branch is currently unfinished.
- `build.rs` + `makehelp/`: help Markdown is compiled into Rust in Cargo's `OUT_DIR` and included by `src/help/mod.rs`.

## Configuration and paths

`gazm.toml` has optional `opts`, `vars`, `checksums`, and `lsp` tables. `opts` is deserialized with kebab-case field names and rejects unknown fields. The config loader sets `assemble_dir` to the config's parent, so relative project, include, and output paths should be reasoned about from that directory.

## Extension points

To add a CPU, update `cpukind.rs`, implement the CPU assembler/frontend modules, and wire parsing/encoding without changing the shared assembler contracts. To add an output, extend `Opts`, the assembler output state, and `assembler/writers.rs`; add a fixture or focused test. To add help, create a Markdown file under `gazm/assets/help/` and let `build.rs` regenerate the embedded index.

## Known limitations

The repository is mid-refactor. Some parser variants contain TODOs, several CPU enum variants are placeholders, and LSP/format commands are not complete. Check the current source and tests before relying on historical notes in `gazm/README.md`.

## Deferred LSP responsiveness work

The LSP currently uses a synchronous `BufRead`/`Write` transport and performs
project analysis inline. This is intentionally unchanged for now. If analysis
latency begins to affect editing responsiveness, migrate incrementally:

1. Keep the assembler pipeline synchronous and move project analysis onto a
   dedicated worker.
2. Send analysis requests through a channel with a monotonically increasing
   generation number.
3. Discard results from older generations so stale diagnostics cannot replace
   newer ones.
4. Keep protocol writes serialized.
5. Only then consider replacing the transport with Tokio and using
   `spawn_blocking` around the synchronous assembler.

Making the assembler itself asynchronous is not part of this plan; its CPU
work should continue to use explicit parallelism where appropriate.
