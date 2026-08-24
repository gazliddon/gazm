# Agent Trail — nvim/gazm integration work (2026-08-23)

For the next agent working in this workspace. This records what was done
in the DeepSeek Harness session that touched gazm, the nvim config, and
the nvim-treesitter migration, plus what remains. The full roadmap also
lives in `../crates/stargate-emu/docs/pickup.md`.

## Done and verified

### 1. gazm-plugin treesitter hook is dual-branch ready (committed)

- `18edbfe` — `lua/gazm/treesitter.lua` supports both nvim-treesitter
  `master` (old `get_parser_configs()` API) and `main` (plain parsers
  table + `User TSUpdate` autocmd, `path`-based install from the local
  `treesitter-gazm` checkout).
- `93b2efe` — plugin cleanup: deleted dead `lua/gazm/text.lua` and
  `test.lua`, `.gazm`-only filetype via `vim.filetype.add`, plenary
  dependency removed from `paths.lua`, README + .gitignore added.
- `gazm.lua` spec (dotfiles `bf42a9f`): `main = 'gazm'`, LSP config,
  plenary dropped. `plugins/sixtyeight.lua` deleted (dead 68xx LSP).

### 2. nvim-treesitter migrated from frozen `master` to `main` (done + verified)

Why: the archived `master` does not support nvim 0.12 — every fenced
code block crashed with `attempt to call method 'range' (a nil value)`
(upstream issue nvim-treesitter#8636). `main` is the supported branch.

What was done:

- Installed `tree-sitter-cli` 0.26.12 via Homebrew. **mason cannot
  install it** (not in the registry) and `main`'s installer needs it for
  every parser (`tree-sitter build` / `tree-sitter generate` in
  `lua/nvim-treesitter/install.lua`). Note: the plain `tree-sitter` brew
  formula is now library-only; use `tree-sitter-cli`.
- Rewrote `~/.config/nvim/lua/plugins/nvim-treesitter.lua`
  (dotfiles commit `7e3f9d9`): `branch = "main"`, new API
  (`require("nvim-treesitter").setup {}` + `.install { ... }`), a
  `FileType` autocmd calling `vim.treesitter.start()` for highlighting
  (moved to core), `norg` dropped (removed upstream), `query_linter`/
  `indent` module options gone. Full file:

```lua
return {
  {
    "nvim-treesitter/nvim-treesitter",
    branch = "main",
    lazy = false,
    build = ":TSUpdate",
    config = function()
      pcall(function()
        require("gazm.treesitter").add_treesitter({})
      end)
      require("nvim-treesitter").setup {}
      require("nvim-treesitter").install {
        "janet_simple", "clojure", "zig", "rust", "lua", "vim", "vimdoc",
        "query", "javascript", "html", "css", "scss", "markdown",
        "markdown_inline", "python", "bash", "typescript", "regex",
        "latex", "svelte", "tsx", "typst", "vue", "gazm",
      }
      vim.api.nvim_create_autocmd("FileType", {
        callback = function()
          pcall(vim.treesitter.start)
        end,
      })
    end,
    dependencies = {
      "nvim-treesitter/playground",
      cmd = { "TSPlaygroundToggle", "TSHighlightCapturesUnderCursor" },
      keys = {
        { "<leader>tp", "<cmd>TSPlaygroundToggle<CR>", desc = "Toggle Treesitter Playground" },
        { "<leader>tc", "<cmd>TSHighlightCapturesUnderCursor<CR>", desc = "Show highlight under cursor" },
      },
    },
  },
}
```

- Ran `:Lazy! sync` headless — checkout is now `main`
  (`e82ef6ae`). Parser installs land in
  `~/.local/share/nvim/site/parser/` (all 24 installed, including
  `gazm.so` built from the local checkout). **Gotcha:** `:TSUpdate` /
  `.install {}` is async with no `TSUpdateSync` on main — keep nvim
  alive (e.g. headless `sleep 240`) or the queue is cut off.

Verified headless with the real config:

```text
MD_FT: markdown   MD_TS: true      # blitter.md: highlighting on, 0 crash errors
GAZM_FT: gazm     GAZM_TS: true    # gazm file highlighting works
GAZM_LSP: { "gazm" }               # gazm LSP attaches
GAZM_CPU: 6809 stargate            # gazm/target handler feeds lualine
```

## Remaining roadmap (for the gazm agent)

Priority order from `../crates/stargate-emu/docs/pickup.md`:

1. **gazm GitHub releases** (Windows/Linux/macOS binaries). Blocker:
   `gazm/Cargo.toml` path-deps on harness crates outside this repo
   (`../../crates/{emu6809,emu6800,grl-sources,grl-utils,grl-symbols,
   grl-eval,unraveler}` — the `gazliddon/crates` repo). Decide between:
   - A) gazm CI checks out both repos (path deps)
   - B) publish the 7 crates to crates.io + `[patch.crates-io]` in the
     dev workspace (recommended)
   - C) vendor crates into this repo (rejected)
   Then a tag-triggered GH Actions workflow (`taiki-e/upload-rust-binary`
   matrix). gazm version is 0.9.16, no platform-specific deps.
2. **Publish gazm-plugin as its own repo**: `git subtree split -P
   gazm-plugin -b plugin-export`, push to a new repo (e.g.
   `gazliddon/gazm.nvim`); the vendored `treesitter-gazm` travels with it.
3. **lazy dev-mode spec** (`plugins/gazm.lua`): local
   `~/development/gazm/gazm-plugin` when present, git otherwise
   (`dev.patterns = { 'gazliddon' }`, `fallback = true`), and auto-detect
   the LSP command (dev `target/release/gazm` -> `gazm` on PATH).
4. **README**: document other-machine setup (`cargo install --path`).

## Small leftovers

- `gazm-plugin/queries/gazm/indents.scm` is empty — fill it and replace
  the `<CR>` indent hack in `lua/gazm/lsp.lua` with `vim.bo.indentexpr`.
- The spec's `<CR>` format-on-Enter mapping is shadowed by the plugin's
  `<CR>` indent mapping (both buffer-local insert mappings).

## Control-flow: design question — compile-time functions (mostly done)

> How do we create functions? Are they macros? In the control structure
> I'd like, for example, make a sin table: get a float sin and then emit
> it as bytes in a way I want — never floats on an 8-bit system. Fns
> don't feel like macros, they just execute during compilation. So that
> implies some kind of floating-point thing in the control code and a
> sin function. What's the best way forward?

**Conclusion already reached:** user-defined functions are NOT macros.
Macros expand to code; a `fn` *executes at assembly time* and produces a
value (a compile-time evaluable function, like Rust `const fn`).

**Status (v0.11.0):** items 1 and 2 are DONE and released. Item 3
(user-defined `fn`) is still open — it's the natural next feature now
that the value model is proven.

1. **DONE — Floats in the evaluator.** Float literals (`3.14`) promote
   arithmetic to f64 in the evaluator; explicit `round()` converts back
   (never silent truncation); a float at an emission boundary is an
   error. `BinaryOp` identity lives in the operator table; int and float
   apply semantics live side by side in the evaluator.
2. **DONE — Builtin compile-time functions.** `sin(x)`, `cos(x)`,
   `round(x)` — and `sizeof(Struct)` for struct totals (the old
   auto-created `Name::size` member is gone; sizes live in
   `asm_out.struct_sizes`). `assert <cond> [, "msg"]` fails the build on
   a false condition (non-fatal, multiple report together) and
   `log <"text" | expr>` prints during assembly. The sin-table use case
   works: `for i in 0..256 { fcb round(sin(i * 2 * 3.14159 / 256) * 127) }`.
3. **OPEN — User-defined `fn`.** A named, parameterized control-flow
   block that evaluates to a value. Bigger design (reuse of the
   sizer-time control flow, return-value mechanics, recursion policy,
   call depth).

**Notes from the build:** loop indices (repeat/for) are per-scope
temporaries — reusing the same index name across loops in one scope
creates-or-reuses instead of erroring. Labels get their PC values as the
sizer walks in source order, so `assert` must come *after* what it
checks (forward label refs in size-time conditions aren't resolvable).


## Environment notes

- DSH sandbox: `workspace-write` allows writes under the session
  workspace only. Workspaces registered in
  `~/.dsh/storages/workspace.json`; `gazm` points at `~/development/gazm`.
- `~/.config` is a symlink to `~/dotfiles/.config` — same files.
- The `gazm` CLI has a `lsp` subcommand (`gazm lsp gazm.toml`); the LSP
  branch is still incomplete per `gazm-plugin/AGENTS.md`.

### Verifying the stargate/robotron fixtures — sandbox write quirk

`~/development/stargate` and `~/development/robotron` are **outside** the
gazm workspace, so under the `workspace-write` sandbox the assembler
cannot write their `roms/` outputs. The build prints
`Misc: Unable to write binary file "<...>/roms/01"` and ends with
`Error: "one or more targets failed"`.

**Trap:** running `sha1sum -c roms.sha1` right after such a build still
reports `OK` — against the **stale** ROMs left by a previous build. The
new binary never wrote anything, so a green checksum there proves
nothing. Do not "verify" with an in-place build under the sandbox.

**Do this instead** — build in a temp copy:

```sh
# stargate (committed tree): clean checkout semantics
rm -rf /tmp/sg && mkdir -p /tmp/sg
cd ~/development/stargate && git ls-files -z | tar --null -T - -cf - | tar -xf - -C /tmp/sg
cd /tmp/sg && ~/development/gazm/target/release/gazm build && sha1sum -c roms.sha1

# robotron (compile-only check; has uncommitted renames, so tar of
# git ls-files would drop them — use a plain copy instead)
rm -rf /tmp/robo && mkdir -p /tmp/robo
cp -R ~/development/robotron/. /tmp/robo/
cd /tmp/robo && ~/development/gazm/target/release/gazm build
```

Notes:
- `Cannot load binary ref file orig/roms/*` errors during the build are
  expected (the original ROM images aren't committed); the byte-identity
  gate is `sha1sum -c roms.sha1`, not the build's exit code.
- Outside the sandbox (a normal shell), in-place builds write the ROMs
  fine; this only bites when a DSH agent runs the build.


## 2026-08-23 — Stargate emulator: nvram, 6821 IRQ model, 6809 core fixes (commit 7b2ee01)

**What works now** (verified against MAME 0.289):
- Empty-CMOS boot matches MAME exactly: validation fail -> re-init -> the
  `$E6B5` error-ack poll (waiting for the Advance/IN2 bit 1 service switch).
  This is CORRECT behaviour, not a bug. MAME with an empty nvram does the
  same.
- With a valid nvram the pass path runs the main loop and reaches the attract
  drawing phase (`$2AC0` sprite plotter; ours at ~14.85M instr, MAME at
  ~11.99M — residual gap is a small beam-phase cycle drift).
- CMOS checksum: half-open `[CC36, CC9E)`, low nibbles, +`$37`, packed across
  `CCA0`(hi)/`CCA1`(lo) as `(cmos[CCA0]<<4)|(cmos[CCA1]&0x0F)`; the `|0xF0`
  write mask is what makes the `<<4` round-trip.

**Fixes in this session:**
- emu6809: postbyte 0x9F `[abs]` was 2 bytes/no indirection — now 4 bytes,
  EA = pointer at operand. Undocumented 0x01 (NEG direct) added (Stargate
  jumps into a JSR operand at $0120 and executes it as NEG). Cycle-count
  corrections (16-bit stores, JMP/JSR ext, CMPX-family imm, direct-page mem
  ops, ORCC/ANDCC, long branches, indexed EA adders).
- stargate-emu bus: 6821 model (data A/ctl A/data B/ctl B at C80C..C80F,
  CA1=scanline>=240, CB1=bit5, flags latch on active edge, level-based IRQ,
  data reads clear flags). Machine: IRQ level-based (no stale latch re-fire).
- IRQ vector is $FFF8/$FFF9 = $9C6B (the beam handler), not SWI.

**How to verify:** `cargo run --release --example passpath <romdir> <nvram> 25000000`
with a MAME-generated nvram (e.g. `/tmp/mnv_test/stargate/nvram`) — the game
draws the attract. `advance` example presses the Advance switch for the
empty-nvram fail path.

**Next steps / open items:**
- The remaining ~0.03% cycle drift shifts the IRQ handler's beam-derived
  value (`$7E`, the `15E3 BLS` branch) and delays the attract entry; compare
  per-instruction cycle totals against the m6809 core.
- Palette: neither MAME (60 s) nor ours (40 M instr) has written C000-C00F —
  the attract colour init happens later in both; the drawn frame is currently
  all-black because the palette is 0.
- The `examples/` dir (advance/passpath/chk/dump/trace/...) is temporary
  debug scaffolding; `passpath.rs` is the useful verification harness.

## 2026-08-23 (later) — Project moved: emulator now at ~/development/williams-emu

The Williams emulator project moved OUT of ~/development/crates into its own
repo: `~/development/williams-emu` (commits: crates `ecaac26`, williams-emu
`f278af8`).

Layout:
- `williams-emu/` root = the machine crate (was stargate-emu): src, docs,
  examples; binaries `williams-emu` (CLI) and `stargate-frame` (renderer).
- `app/` = the interactive winit/OpenGL/ImGui frontend (was stargate-app);
  its build.rs compiles the tree-sitter gazm parser from
  `../../gazm/gazm-plugin/treesitter-gazm/src` (path unchanged).
- `debugger/` = the ratatui terminal debugger (was stargate-debug).

Dependencies stayed in ~/development/crates and are referenced by path:
williams-emu -> `../crates/{emu6809,emucore}`; app -> `../../crates/{wms-sound,grl-sources}`.
Running the app: `cargo run --release -p app -- ~/development/stargate/roms`
(F3 = Advance service switch). Verification harness:
`cargo run --release --example passpath -- <roms> <nvram> <count>`.
