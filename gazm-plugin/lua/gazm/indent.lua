--- Gazm-specific indentation.
---
--- Replaces the old buffer-local `<CR>` mapping in `gazm.lsp` with a proper
--- `indentexpr`. Gazm's layout rules:
---
---   * label definitions (`name:` / `@name:` / `!name:`) start at column 0
---   * statements (opcodes, commands, macros) start at the opcode column
---     (16 spaces, zero-based column 16)
---   * struct and macro bodies indent one level past the opcode column
---   * the closing brace of a struct/macro body returns to the opcode column
---
--- The tree-sitter grammar is used to detect struct/macro bodies when the
--- parser is available; the fixed-column rules still apply otherwise.

local M = {}

-- Visible column 17, zero-based column 16.
local OPCODE_COLUMN = 16

-- Node types whose bodies indent past the opcode column.
local BLOCK_TYPES = {
    struct_def = true,
    macro_body = true,
}

local function line_at(lnum)
    return vim.api.nvim_buf_get_lines(0, lnum - 1, lnum, false)[1] or ''
end

-- First node at (row, first-nonspace-column) if a gazm parser is attached.
-- Returns nil when the parser is missing or not yet parsed, so callers can
-- fall back to the fixed-column heuristics.
local function node_at_first_nonspace(row)
    local line = line_at(row + 1)
    local first = line:find('%S')
    local col = first and (first - 1) or 0

    local ok, node = pcall(vim.treesitter.get_node, {
        bufnr = 0,
        pos = { row, col },
        lang = 'gazm',
    })
    if not ok or not node then
        return nil
    end
    return node
end

local function inside_block(row)
    local node = node_at_first_nonspace(row)
    if not node then
        return nil
    end
    while node do
        if BLOCK_TYPES[node:type()] then
            -- The block's own opening line (`struct x {`) is a statement,
            -- not a body line: only lines strictly after the block start
            -- are considered inside the body.
            local start_row = node:start()
            if start_row < row then
                return node:type()
            end
            return nil
        end
        node = node:parent()
    end
    return false
end

-- Matches a label definition at the start of the line:
--   `name:`  `@name:`  `!name:`  (optionally indented)
local LABEL_DEF = '^%s*[@!]?[A-Za-z_][A-Za-z0-9_.]*%s*:'

function M.expr()
    local lnum = vim.v.lnum
    local line = line_at(lnum)
    local block = inside_block(lnum - 1)

    if block then
        if line:match('^%s*}') then
            return OPCODE_COLUMN
        end
        return OPCODE_COLUMN + vim.bo.shiftwidth
    end

    if line:match(LABEL_DEF) then
        return 0
    end

    return OPCODE_COLUMN
end

return M
