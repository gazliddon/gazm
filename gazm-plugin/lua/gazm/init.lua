--- @TODO Check to make sure plenary is installed

local settings = require("gazm.settings")
local lsp = require("gazm.lsp")
local ts = require("gazm.treesitter")

local M = {}

function M.setup(opts)
    if opts then
        settings.set(opts)
    end

    opts = settings.current

    ts.add_treesitter(opts)
    lsp.init(opts)
end

return M




