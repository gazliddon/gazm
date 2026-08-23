local M = {}

-- Whether the custom `gazm/target` notification handler is installed.
-- Module-local so it is re-registered if the plugin is reloaded.
local target_handler_installed = false

local function setup_newline_indent(bufnr)
    vim.keymap.set('i', '<CR>', function()
        local keys = vim.api.nvim_replace_termcodes('<CR>', true, false, true)
        vim.api.nvim_feedkeys(keys, 'n', false)
        vim.schedule(function()
            local cursor = vim.api.nvim_win_get_cursor(0)
            local row = cursor[1]
            local line = vim.api.nvim_get_current_line()
            if not line:match('^%s*$') then
                return
            end

            -- Gazm keeps opcodes at visible column 17 (zero-based column 16).
            local indent = string.rep(' ', 16)
            vim.api.nvim_buf_set_lines(bufnr, row - 1, row, false, { indent })
            vim.api.nvim_win_set_cursor(0, { row, #indent })
        end)
    end, {
        buffer = bufnr,
        desc = 'Indent new Gazm statement',
    })
end

function M.init(opts)
    if not target_handler_installed then
        vim.lsp.handlers['gazm/target'] = function(_, params)
            if not params or not params.uri then
                return
            end
            local bufnr = vim.uri_to_bufnr(params.uri)
            if vim.api.nvim_buf_is_loaded(bufnr) then
                vim.b[bufnr].gazm_target = params.target or ''
                vim.b[bufnr].gazm_cpu = params.cpu or ''
                vim.cmd('redrawstatus')
            end
        end
        target_handler_installed = true
    end

    local ok, lspconfig = pcall(require, 'lspconfig')

    if ok then
        local lsp_opts = opts.lsp

        local configs = require("lspconfig.configs")
        local util = require('lspconfig/util')
        local cmd = { lsp_opts.command, 'lsp', lsp_opts.config }

        if not configs.gazm then
            configs.gazm = {
                default_config = {
                    cmd = cmd,
                    filetypes = { 'gazm' },
                    root_dir = util.root_pattern(lsp_opts.root_pattern),
                    settings = {},
                },
            }
        end

        lspconfig.gazm.setup {
            cmd = cmd,
            filetypes = { 'gazm' },
            root_dir = util.root_pattern(lsp_opts.root_pattern),
            on_attach = function(client, bufnr)
                if lsp_opts.on_attach then
                    lsp_opts.on_attach(client, bufnr)
                end
                setup_newline_indent(bufnr)
            end,
        }
    else
        vim.notify("gazm: nvim-lspconfig is not installed; LSP disabled", vim.log.levels.WARN)
    end
end

return M
