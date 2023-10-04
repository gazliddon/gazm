local M = {}

function M.init(opts)
    local ok, lspconfig = pcall(require, 'lspconfig')

    if ok then
        local lsp_opts = opts.lsp

        local configs = require("lspconfig.configs")
        local util = require('lspconfig/util')
        local cmd = { 'gazm', 'lsp', 'gazm.toml' }

        if not configs.gazm then
            configs.gazm = {
                default_config = {
                    cmd = cmd,
                    filetypes = { 'gazm' },
                    root_dir = util.root_pattern("gazm.toml"),
                    settings = {},
                },
            }
        end

        lspconfig.gazm.setup {
            on_attach = lsp_opts.on_attach
        }
    end
end

return M
