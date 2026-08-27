# gazm.nvim

Neovim integration for the [gazm](https://github.com/gazliddon/gazm)
assembler toolchain. Provides filetype detection for `.gazm` files, a
tree-sitter grammar (vendored under `treesitter-gazm/`), an LSP client
for the `gazm` language server, and gazm-specific editor conveniences.

## Requirements

- Neovim 0.10+
- [gazm](https://github.com/gazliddon/gazm) CLI with the `lsp` command
  (the LSP branch is still incomplete; the plugin degrades gracefully)
- `nvim-treesitter` (for the gazm parser) and `nvim-lspconfig` (for the
  LSP client)

## Installation

With [lazy.nvim](https://github.com/folke/lazy.nvim):

```lua
{
  dir = '~/development/gazm/gazm-plugin',
  main = 'gazm',
  opts = {
    lsp = {
      command = 'gazm', -- or an absolute path to a debug build
      config = 'gazm.toml',
      root_pattern = 'gazm.toml',
    },
  },
  dependencies = {
    'nvim-treesitter/nvim-treesitter',
  },
}
```

`*.gazm` files always map to the `gazm` filetype. The common assembler
extensions (`.s`, `.asm`, `.gas`, `.68`, `.inc`) map to `gazm` too, but
only when the file lives inside a gazm project (a directory tree containing
a `gazm.toml`); outside one they keep whatever filetype other plugins
assign, so unrelated assembler projects are not hijacked.

## Options

Passed to `setup` via lazy's `opts` (or `require('gazm').setup(...)`):

| Key            | Default      | Description                        |
|----------------|--------------|------------------------------------|
| `lsp.command`  | `"gazm"`     | Path to the gazm binary            |
| `lsp.config`   | `"gazm.toml"`| Config file passed to the LSP      |
| `lsp.root_pattern` | `"gazm.toml"` | LSP root dir marker            |
| `lsp.on_attach`| `function() end` | Extra `on_attach` callback     |

## Features

- `gazm` filetype for `.gazm` files (and for `.s`/`.asm`/`.gas`/`.68`/`.inc`
  inside a gazm project); `;` comments via vim-commentary
- Tree-sitter highlighting (one CPU-agnostic grammar for all targets),
  injections, and indentation queries
- LSP: `gazm/target` notifications publish the whole project file→target
  map; the handler feeds the statusline (`b.gazm_cpu`, `b.gazm_target`)
  and keeps the map for files opened later. Query it with
  `require('gazm.lsp').cpu_for_buf()` / `target_for_buf()`. Semantic
  tokens (CPU-accurate keywords/registers) are layered on top of the
  grammar highlights automatically
- Project detection (`require('gazm.project').in_project(path)`,
  `.root(path)`) is shared by ftdetect and user configs
- New statements start at the gazm opcode column (16 spaces); labels at
  column 0 and struct/macro bodies one level deeper are handled by
  `indentexpr` (`lua/gazm/indent.lua`) plus the `indents.scm` query

## Development

See `AGENTS.md` for conventions. The tree-sitter grammar lives in
`treesitter-gazm/` — edit `grammar.js` and regenerate; never hand-edit
`src/parser.c` or the generated bindings.

## License

MIT
