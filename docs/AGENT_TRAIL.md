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

## Environment notes

- DSH sandbox: `workspace-write` allows writes under the session
  workspace only. Workspaces registered in
  `~/.dsh/storages/workspace.json`; `gazm` points at `~/development/gazm`.
- `~/.config` is a symlink to `~/dotfiles/.config` — same files.
- The `gazm` CLI has a `lsp` subcommand (`gazm lsp gazm.toml`); the LSP
  branch is still incomplete per `gazm-plugin/AGENTS.md`.
