# Editor plugin notes

`gazm-plugin` is Neovim integration written in Lua, with a Tree-sitter grammar/package under `treesitter-gazm/`.

- Keep Neovim setup behavior in `lua/gazm/`; keep grammar changes in `treesitter-gazm/grammar.js` and update corpus cases when syntax changes.
- Do not hand-edit generated parser outputs (`src/parser.c`, node-types, or bindings) unless the grammar toolchain requires it; regenerate them using the package's documented Tree-sitter workflow.
- The plugin can refer to the `gazm` CLI/LSP, but the current CLI LSP branch is incomplete; preserve graceful behavior when it is unavailable.
