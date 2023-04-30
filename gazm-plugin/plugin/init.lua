local tsparsers = require "nvim-treesitter.parsers"

if tsparsers ~= nil then
    local parser_config = tsparsers.get_parser_configs()
    parser_config.gazm = {
        install_info = {
            url = "~/development/gazm/treesitter-gazm", -- local path or git repo
            files = { "src/parser.c" },
            -- optional entries:
            -- branch = "main", -- default branch in case of git repo if different from master
            generate_requires_npm = false, -- if stand-alone parser without npm dependencies
            -- requires_generate_from_grammar = false, -- if folder contains pre-generated src/parser.c
        },
        -- filetype = "68", -- if filetype does not match the parser name
    }
end
