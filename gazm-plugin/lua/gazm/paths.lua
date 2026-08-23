local M = {}

-- Absolute path to the plugin root (parent of `lua/`).
local function find_plugin_dir()
    -- debug.getinfo gives the full path to this file:
    --   <plugin>/lua/gazm/paths.lua  ->  <plugin>
    local src = debug.getinfo(1, 'S').source:sub(2)
    return vim.fn.fnamemodify(src, ':p:h:h:h')
end

M.plugin_dir = find_plugin_dir()

return M
