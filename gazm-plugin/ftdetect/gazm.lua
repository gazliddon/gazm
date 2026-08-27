-- ftdetect/gazm.lua
--
-- The `.gazm` extension always maps to the gazm filetype.  The common
-- assembler extensions (`.s`, `.asm`, `.gas`, `.68`, `.inc`) are only
-- claimed when the file lives inside a gazm project — a directory tree that
-- contains a `gazm.toml`; outside one the detection function returns nil
-- and other plugins' mappings keep working, so unrelated assembler projects
-- are not hijacked.
--
-- The entries use a positive priority so they win over generic `.s`/`.asm`
-- mappings from other plugins *inside* a gazm project.  Once the LSP is
-- attached it refines the picture per file (`gazm/target` notifications
-- carry the target name and CPU), but filetype detection has to decide
-- before the LSP starts.

local ok, project = pcall(require, 'gazm.project')

local function gazm_in_project(path, _bufnr)
    if ok and project.in_project(path) then
        return 'gazm'
    end
end

-- Ambiguous extensions: only gazm inside a gazm project.
local ambiguous = { 's', 'asm', 'gas', '68', 'inc' }
local patterns = {}
for _, ext in ipairs(ambiguous) do
    patterns['.*%.' .. ext] = { gazm_in_project, { priority = 10 } }
end

vim.filetype.add({
    extension = {
        gazm = 'gazm',
    },
    pattern = patterns,
})
