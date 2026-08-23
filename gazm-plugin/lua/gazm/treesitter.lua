local M = {}

local paths = require("gazm.paths")

-- Register the gazm parser with nvim-treesitter.
--
-- Works with both:
--   * the frozen `master` branch (old API: parsers.get_parser_configs())
--   * the `main` branch (parsers.lua is a plain table; custom parsers are
--     registered in a `User TSUpdate` autocmd so :TSInstall/:TSUpdate see
--     them, using `path` for a local checkout)
function M.add_treesitter(opts)
    local ok, tsparsers = pcall(require, "nvim-treesitter.parsers")

    if ok then
        local ts_dir = vim.fs.joinpath(paths.plugin_dir, 'treesitter-gazm')
        vim.opt.rtp:append(paths.plugin_dir)

        if type(tsparsers.get_parser_configs) == 'function' then
            -- frozen master branch
            local parser_config = tsparsers.get_parser_configs()

            parser_config.gazm = {
                install_info = {
                    url = ts_dir,
                    files = { "src/parser.c" },
                    generate_requires_npm = false, -- stand-alone parser without npm deps
                },
            }
        else
            -- main branch: plain parser table; add gazm now and on every
            -- TSUpdate so installs from the local checkout keep working
            local function add_gazm()
                local parsers = require("nvim-treesitter.parsers")
                if type(parsers) == 'table' then
                    parsers.gazm = {
                        install_info = {
                            path = ts_dir, -- local checkout, used as-is
                        },
                    }
                end
            end

            add_gazm()
            vim.api.nvim_create_autocmd("User", {
                pattern = "TSUpdate",
                callback = add_gazm,
            })
        end
    end
end

return M
