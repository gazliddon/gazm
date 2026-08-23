local M = {}

-- Whether the custom `gazm/target` notification handler is installed.
-- Module-local so it is re-registered if the plugin is reloaded.
local target_handler_installed = false

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
            end,
        }
    else
        vim.notify("gazm: nvim-lspconfig is not installed; LSP disabled", vim.log.levels.WARN)
    end
end

return M
