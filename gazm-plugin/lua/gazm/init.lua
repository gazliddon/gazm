local settings = require("gazm.settings")
local lsp = require("gazm.lsp")
local ts = require("gazm.treesitter")

local M = {}

function M.setup(opts)
    if opts then
        settings.set(opts)
    end

    local cfg = settings.current

    ts.add_treesitter()
    lsp.init(cfg)
end

return M
