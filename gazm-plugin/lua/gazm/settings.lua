--- Hold the settings for the gazm plugin

local M = {}

local DEFAULT_SETTINGS = {
    lsp = {
        onattach = function()
        end
    }
}

M._DEFAULT_SETTINGS = DEFAULT_SETTINGS
M.current = M._DEFAULT_SETTINGS

function M.set(opts)
    M.current = vim.tbl_deep_extend("force", vim.deepcopy(M.current), opts)
    return M.current
end

return M
