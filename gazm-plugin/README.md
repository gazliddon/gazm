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

Only `*.gazm` files are associated with the `gazm` filetype.

## Options

Passed to `setup` via lazy's `opts` (or `require('gazm').setup(...)`):

| Key            | Default      | Description                        |
|----------------|--------------|------------------------------------|
| `lsp.command`  | `"gazm"`     | Path to the gazm binary            |
| `lsp.config`   | `"gazm.toml"`| Config file passed to the LSP      |
| `lsp.root_pattern` | `"gazm.toml"` | LSP root dir marker            |
| `lsp.on_attach`| `function() end` | Extra `on_attach` callback     |

## Features

- `gazm` filetype for `.gazm` files; `;` comments via vim-commentary
- Tree-sitter highlighting, injections, and indentation queries
- LSP: `gazm/target` notification handler feeds the statusline
  (`b.gazm_cpu`, `b.gazm_target`)
- New statements start at the gazm opcode column (16 spaces)

## Development

See `AGENTS.md` for conventions. The tree-sitter grammar lives in
`treesitter-gazm/` — edit `grammar.js` and regenerate; never hand-edit
`src/parser.c` or the generated bindings.

## License

MIT
