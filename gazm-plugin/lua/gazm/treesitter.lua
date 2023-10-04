local M = {}

local paths = require("gazm.paths")

function M.add_treesitter(opts)

    local ok,tsparsers = pcall(require,"nvim-treesitter.parsers")

    if ok ~= nil then
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

return M
