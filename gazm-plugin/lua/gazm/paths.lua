local M = {}

local Path = require("plenary.path")

local function find_plugin_dir()
    local this_file = debug.getinfo(1).source:sub(2)
    local p = Path.new(this_file):parent()
    return p:find_upwards("ftdetect"):parent()
end

M.plugin_dir_path = find_plugin_dir()
M.plugin_dir = tostring(M.plugin_dir_path)

return M
