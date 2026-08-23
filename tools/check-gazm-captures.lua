-- Apply the gazm highlights query to a sample file and report which
-- captures fire (and where).
--
-- Usage: nvim --headless -u NONE -i NONE --cmd "set rtp+=<plugin dir>" \
--            -l tools/check-gazm-captures.lua
--
-- Sample file lives at /tmp/sample.gazm; override with GAZM_SAMPLE.

local file = vim.env.GAZM_SAMPLE or '/tmp/sample.gazm'
local buf = vim.api.nvim_create_buf(false, true)
vim.api.nvim_buf_set_lines(buf, 0, -1, false, vim.fn.readfile(file))
vim.bo[buf].filetype = 'gazm'
vim.api.nvim_set_current_buf(buf)

local ok, parser = pcall(vim.treesitter.get_parser, buf, 'gazm')
if not ok then
    print('PARSER FAILED: ' .. tostring(parser))
    return
end
parser:parse()
local root = parser:parse()[1]:root()

local query = vim.treesitter.query.get('gazm', 'highlights')
assert(query, 'highlights query missing')

local found = {}
for id, node, _ in query:iter_captures(root, buf, 0, -1) do
    -- query.captures[id] is the capture name WITHOUT the leading '@'
    local name = query.captures[id]
    found[name] = (found[name] or 0) + 1
end

print('PARSER OK, root=' .. root:type())
-- Report with the '@' prefix, matching how captures appear in the query file
-- and in :TSHighlightCapturesUnderCursor.
local wanted = { 'keyword.directive', 'function.builtin', 'label', 'comment.documentation', 'string', 'number' }
for _, w in ipairs(wanted) do
    local n = found[w]
    print(string.format('capture @%-28s count=%s', w, n and tostring(n) or '0 (MISSING)'))
end
print('total captures: ' .. vim.inspect(found))
