local M = {}

function M.setup()
    local tsparsers = require "nvim-treesitter.parsers"
    local Path = require("plenary.path")

    local function get_script_file()
        return Path.new(debug.getinfo(1).source:sub(2))
    end

    local function plugin_dir()
        return get_script_file():parent():parent()
    end

    local pdir = plugin_dir()
    local ts_url = tostring(pdir:joinpath('treesitter-gazm'))

    if tsparsers ~= nil then
        vim.opt.rtp:append(tostring(pdir))

        local parser_config = tsparsers.get_parser_configs()

        parser_config.gazm = {
            install_info = {
                url = ts_url,
                files = { "src/parser.c" },
                -- optional entries:
                -- branch = "main", -- default branch in case of git repo if different from master
                generate_requires_npm = false, -- if stand-alone parser without npm dependencies
                -- requires_generate_from_grammar = false, -- if folder contains pre-generated src/parser.c
            },
        }
    end
end

return M


