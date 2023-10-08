local M = {}

local function get_nodes_iter(query_string)
    local parse = vim.treesitter.query.parse;
    local parser = require('nvim-treesitter.parsers').get_parser()

    local ok, query = pcall(parse, parser:lang(), query_string)

    if not ok then
        return false, "query failed", false
    end

    local tree = parser:parse()[1]

    return true, tree:root(), query
end

local function indent_nodes(root, query, column)
    local nodes = query:iter_captures(root, 0)
    for _, node in nodes do
        local line, col, endline, endcol = node:range()
        local adj = column - col;

        if adj > 0 then
            local spaces = string.rep(" ", adj)
            vim.api.nvim_buf_set_text(0, line, col, line, col, { spaces })
        elseif adj < 0 then
            local grab = vim.api.nvim_buf_get_text(0, line, col + adj, line, col, {})
            if grab[1]:match("^%s+$") then
                vim.api.nvim_buf_set_text(0, line, col + adj, line, col, {})
            end
        end
    end
end

local function iter_nodes(root, query, func)
    local nodes = query:iter_captures(root, 0)
    for _, node in nodes do
        func(node)
    end
end

local function indent_node(node, column)
    local line, col, endline, endcol = node:range()
    local adj = column - col;

    if adj > 0 then
        local spaces = string.rep(" ", adj)
        vim.api.nvim_buf_set_text(0, line, col, line, col, { spaces })
    elseif adj < 0 then
        local grab = vim.api.nvim_buf_get_text(0, line, col + adj, line, col, {})
        if grab[1]:match("^%s+$") then
            vim.api.nvim_buf_set_text(0, line, col + adj, line, col, {})
        end
    end
end

local function iter_query(query_text, func)
    local ok, root, query = get_nodes_iter(query_text)

    if not ok then
        return false, "invalid query"
    end

    iter_nodes(root, query, func)
end

function M.hl()
    local opcode_col = 16
    local operand_col = opcode_col + 7
    local comment_col = 44

    iter_query('(mnemonic) @mnemonic', function(node)
        local line, col, endline, endcol = node:range()
        local grab = vim.api.nvim_buf_get_text(0, line, col, endline, endcol, {})
        vim.api.nvim_buf_set_text(0, line, col, endline, endcol, { grab[1] })
    end)

    iter_query('(mnemonic) @mnemonic',function(node)
        indent_node(node, opcode_col)
    end)

    iter_query('(opcode) @opcode', function(node)
        local line, col, endline, endcol = node:range()
        local sib = node:next_sibling()
        if sib then
            local sib_line = sib:range()
            if sib_line == line and sib:type() == 'comment' then
                indent_node(sib, comment_col)
            end
        end
    end)

    iter_query('(macro) @macro', function(node)
        indent_node(node, opcode_col)
    end)

    iter_query('(operand) @operand', function(node)
        indent_node(node, operand_col)
    end)

    iter_query('(reg_set) @reg_set', function(node)
        indent_node(node, operand_col)
    end)

    iter_query('(reg_xfer) @reg_xfer', function(node)
        indent_node(node, operand_col)
    end)
end

return M
