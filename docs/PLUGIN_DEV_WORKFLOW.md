# Developing and testing the gazm lazy dev-mode spec

This workflow covers the lazy.nvim plugin spec in the user dotfiles
(`~/.config/nvim/lua/plugins/gazm.lua` and `~/.config/nvim/lua/gaz/lazy.lua`)
that loads `gazm-plugin` in **dev mode**: a local checkout is used while it
exists, otherwise lazy falls back to the published git repo.

## What the spec does

- `plugins/gazm.lua` declares the plugin by **url**
  (`https://github.com/gazliddon/gazm.nvim`) with `main = 'gazm'`, and
  auto-detects the LSP server command:
  1. `$GAZM_DEV_ROOT/target/release/gazm` (default dev root
     `~/development/gazm`) if executable
  2. `$GAZM_DEV_ROOT/target/debug/gazm` if executable
  3. `gazm` from `PATH` otherwise
- `gaz/lazy.lua` enables lazy dev mode:
  ```lua
  dev = {
    patterns = { "gazliddon" },
    fallback = true,
    path = function(plugin)
      if plugin.name == "gazm.nvim" then
        return vim.fs.normalize("~/development/gazm/gazm-plugin")
      end
      return vim.fs.normalize("~/projects/" .. plugin.name)
    end,
  }
  ```
  Any plugin whose url contains `gazliddon` is resolved from
  `dev.path(plugin)` when that directory exists, and falls back to the git
  url when it does not.

## The test harness

`tools/test-gazm-lazy-spec.lua` is a headless nvim script that loads the
**real** spec files from the dotfiles and asserts:

1. Spec structure (`main`, `url`, dependencies, `opts.lsp.command` type).
2. LSP command auto-detection, by pointing `GAZM_DEV_ROOT` at a scratch
   directory:
   - release build present -> release wins
   - only debug build present -> debug wins
   - no build -> `gazm` on PATH
   - the real environment's command is executable
3. lazy dev-mode resolution, using lazy.nvim's own spec machinery:
   - local checkout present -> `dev = true`, dir is the checkout
   - checkout missing -> `dev = false`, dir under the lazy root (git fallback)
4. The real `gaz/lazy.lua` contains the expected `dev` block.

Run it from the repository root:

```sh
nvim --headless -u NONE -l tools/test-gazm-lazy-spec.lua
```

Expected output ends with `N passed, 0 failed`. A failing check exits with a
non-zero status.

## Full smoke test with the real nvim config

Boot the actual config headless and confirm the plugin resolves to the local
checkout and the LSP command is the dev build:

```sh
XDG_CACHE_HOME=/tmp/nvim-cache-test \
nvim --headless -u ~/.config/nvim/init.lua \
  -c 'lua local p=require("lazy.core.config").plugins["gazm.nvim"]; print("dir="..p.dir); print("dev="..tostring(p.dev)); print("lsp="..p.opts.lsp.command); cquit()'
```

Expected:

```text
dir=/Users/<you>/development/gazm/gazm-plugin
dev=true
lsp=/Users/<you>/development/gazm/target/release/gazm
```

(`XDG_CACHE_HOME` is redirected so the sandbox can write the luac cache;
interactive machines usually do not need it.)

## To simulate the git-fallback branch

The fallback only engages when the local checkout is absent. To observe it
without touching the real checkout, the harness's `resolve_gazm(false)` check
already exercises lazy's real resolution logic against a scratch dir. To see
it in the real config:

1. Temporarily rename the checkout:
   ```sh
   mv ~/development/gazm/gazm-plugin ~/development/gazm/gazm-plugin.hidden
   ```
2. Run the smoke test above; `dev` should now print `false` and `dir` should
   point at `.../lazy/gazm.nvim`.
3. Restore:
   ```sh
   mv ~/development/gazm/gazm-plugin.hidden ~/development/gazm/gazm-plugin
   ```

## Iterating on the plugin itself

While the checkout exists, editing `gazm-plugin/` is picked up on the next
`nvim` start (lazy `dev` mode does not require reinstalling). After grammar
changes, regenerate and test as described in `gazm-plugin/AGENTS.md`:

```sh
cd gazm-plugin/treesitter-gazm
npm install          # once
npm test             # tree-sitter corpus tests
npm run build        # regenerates src/parser.c, grammar.json, node-types.json
```

Then re-run the harness and smoke test to confirm the spec still resolves.

## Prerequisites

- `lazy.nvim` installed (any path on `rtp`; the harness reads it from
  `stdpath('data')/lazy/lazy.nvim`).
- The dotfiles `plugins/gazm.lua` + `gaz/lazy.lua` present at
  `~/.config/nvim/...` (symlinked to `~/dotfiles/.config/nvim` here).
- A `gazm` binary either on `PATH` or at
  `~/development/gazm/target/{release,debug}/gazm`.
