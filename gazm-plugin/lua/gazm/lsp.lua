local M = {}

-- Whether the custom `gazm/target` notification handler is installed.
-- Module-local so it is re-registered if the plugin is reloaded.
local target_handler_installed = false

-- uri -> { target = string, cpu = string }
-- The server publishes the whole project map — every source file of every
-- target — as `gazm/target` notifications, so we learn the CPU of files
-- that are not open yet.  Keep the map so buffers opened later can be
-- tagged with their target/cpu.
local target_map = {}

local function buf_uri(bufnr)
    if vim.api.nvim_buf_get_name(bufnr) == '' then
        return nil
    end
    return vim.uri_from_bufnr(bufnr)
end

-- Apply the stored map entry to a buffer's `b:` variables (idempotent).
local function apply_map_to_buf(bufnr)
    local uri = buf_uri(bufnr)
    local entry = uri and target_map[uri]
    if entry then
        vim.b[bufnr].gazm_target = entry.target
        vim.b[bufnr].gazm_cpu = entry.cpu
    end
end

--- CPU the LSP reported for `bufnr` (current buffer by default), or nil
--- when unknown (LSP not running / file not part of any target yet).
function M.cpu_for_buf(bufnr)
    bufnr = bufnr or vim.api.nvim_get_current_buf()
    local uri = buf_uri(bufnr)
    local entry = uri and target_map[uri]
    if entry and entry.cpu ~= '' then
        return entry.cpu
    end
    return vim.b[bufnr] and vim.b[bufnr].gazm_cpu or nil
end

--- Target name the LSP reported for `bufnr` (current buffer by default),
--- or nil when unknown.
function M.target_for_buf(bufnr)
    bufnr = bufnr or vim.api.nvim_get_current_buf()
    local uri = buf_uri(bufnr)
    local entry = uri and target_map[uri]
    if entry and entry.target ~= '' then
        return entry.target
    end
    return vim.b[bufnr] and vim.b[bufnr].gazm_target or nil
end

function M.init(opts)
    if not target_handler_installed then
        vim.lsp.handlers['gazm/target'] = function(_, params)
            if not params or not params.uri then
                return
            end
            target_map[params.uri] = {
                target = params.target or '',
                cpu = params.cpu or '',
            }
            local bufnr = vim.uri_to_bufnr(params.uri)
            if vim.api.nvim_buf_is_loaded(bufnr) then
                apply_map_to_buf(bufnr)
                vim.cmd('redrawstatus')
            end
        end

        -- Buffers that become gazm after the server already published
        -- their map entry (opened later, or ftdetect ran after attach)
        -- are tagged as soon as the filetype is set.
        vim.api.nvim_create_autocmd('FileType', {
            pattern = 'gazm',
            callback = function(args)
                apply_map_to_buf(args.buf)
            end,
        })

        target_handler_installed = true
    end

    local ok, lspconfig = pcall(require, 'lspconfig')

    if ok then
        local lsp_opts = opts.lsp

        local configs = require("lspconfig.configs")
        local util = require('lspconfig/util')
        local cmd = { lsp_opts.command, 'lsp', lsp_opts.config }

        if not configs.gazm then
            configs.gazm = {
                default_config = {
                    cmd = cmd,
                    filetypes = { 'gazm' },
                    root_dir = util.root_pattern(lsp_opts.root_pattern),
                    settings = {},
                },
            }
        end

        lspconfig.gazm.setup {
            cmd = cmd,
            filetypes = { 'gazm' },
            root_dir = util.root_pattern(lsp_opts.root_pattern),
            on_attach = function(client, bufnr)
                if lsp_opts.on_attach then
                    lsp_opts.on_attach(client, bufnr)
                end
            end,
        }
    else
        vim.notify("gazm: nvim-lspconfig is not installed; LSP disabled", vim.log.levels.WARN)
    end
end

return M
