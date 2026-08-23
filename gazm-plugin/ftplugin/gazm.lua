-- Gazm uses semicolons for both line and inline comments.  This is consumed
-- by vim-commentary (`gcc`, `gc`, and `gcap`) and by Vim's comment-aware text
-- operations.
vim.opt_local.commentstring = "; %s"
vim.opt_local.comments = ":;"

-- Gazm indentation: labels at column 0, statements at the opcode column
-- (16 spaces), struct/macro bodies one level past it.  Replaces the old
-- `<CR>` insert mapping in `gazm.lsp` with a proper `indentexpr`.
vim.opt_local.indentexpr = "v:lua.require('gazm.indent').expr()"
