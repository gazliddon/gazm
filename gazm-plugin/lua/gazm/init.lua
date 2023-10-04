
local settings = require("gazm.settings")
local paths = require("gazm.paths")
local lsp = require("gazm.lsp")

local M = {}

local function add_treesitter(opts)
    local tsparsers = require("nvim-treesitter.parsers")

    if tsparsers ~= nil then
        local ts_url = tostring(paths.plugin_dir_path:joinpath('treesitter-gazm'))
        vim.opt.rtp:append(paths.plugin_dir)

        local parser_config = tsparsers.get_parser_configs()

        parser_config.gazm = {
            install_info = {
                url = ts_url,
                files = { "src/parser.c" },
                generate_requires_npm = false, -- if stand-alone parser without npm dependencies
            },
        }
    end
end

function M.setup(opts)
    if opts then
        settings.set(opts)
    end

    opts = settings.current

    add_treesitter(opts)
    lsp.init(opts)
end

return M




