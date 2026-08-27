-- gazm/project.lua
--
-- Project detection: a directory tree is a gazm project when it contains a
-- `gazm.toml`.  Used by ftdetect to decide whether an ambiguous assembler
-- file (`.s`, `.asm`, `.gas`, ...) belongs to a gazm project, and available
-- to user configs for the same question.

local M = {}

-- dir -> boolean, memoised for the session (filesystem walks are only done
-- once per directory).
local cache = {}

--- Is `path` (a file or a directory) inside a gazm project?
--- Walks upward from the file's directory looking for `gazm.toml`.
function M.in_project(path)
    if type(path) ~= 'string' or path == '' then
        return false
    end
    local dir = vim.fs.dirname(path)
    if dir == nil or dir == '' then
        dir = '.'
    end
    local cached = cache[dir]
    if cached ~= nil then
        return cached
    end
    local found = #vim.fs.find('gazm.toml', { upward = true, path = dir, limit = 1 }) > 0
    cache[dir] = found
    return found
end

--- Directory containing the project's `gazm.toml` (nil when `path` is not
--- inside a gazm project).
function M.root(path)
    if type(path) ~= 'string' or path == '' then
        return nil
    end
    local dir = vim.fs.dirname(path)
    if dir == nil or dir == '' then
        dir = '.'
    end
    local found = vim.fs.find('gazm.toml', { upward = true, path = dir, limit = 1 })[1]
    if not found then
        return nil
    end
    return vim.fs.dirname(found)
end

--- Forget cached results (e.g. after creating/removing a `gazm.toml`).
function M.clear_cache()
    cache = {}
end

return M
