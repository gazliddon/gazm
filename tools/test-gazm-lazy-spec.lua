-- Test harness for the gazm lazy dev-mode spec.
--
-- Validates that `~/.config/nvim/lua/plugins/gazm.lua` and the `dev` block in
-- `~/.config/nvim/lua/gaz/lazy.lua` behave as intended:
--
--   1. The spec resolves the plugin name/url/main correctly.
--   2. LSP command auto-detection: a local dev build of the gazm binary
--      (target/release, then target/debug under GAZM_DEV_ROOT) is preferred,
--      falling back to `gazm` on PATH.
--   3. lazy dev mode: while the local checkout exists the plugin is loaded
--      from it (`dev = true`); when it is missing lazy falls back to git
--      (`dev = false`, dir under the lazy root).
--
-- Usage:
--   nvim --headless -u NONE -l tools/test-gazm-lazy-spec.lua
--
-- Requires lazy.nvim to be installed (it is read from `stdpath('data')`).

local lazypath = vim.fn.stdpath('data') .. '/lazy/lazy.nvim'
if vim.fn.isdirectory(lazypath) ~= 1 then
    error('lazy.nvim not found at ' .. lazypath .. '; run your normal nvim config once first')
end
vim.opt.rtp:prepend(lazypath)

local spec_file = vim.fs.normalize('~/.config/nvim/lua/plugins/gazm.lua')
local lazy_file = vim.fs.normalize('~/.config/nvim/lua/gaz/lazy.lua')

local pass = 0
local fail = 0

local function check(name, cond, detail)
    if cond then
        pass = pass + 1
        print('PASS  ' .. name)
    else
        fail = fail + 1
        print('FAIL  ' .. name .. (detail and ('  -- ' .. detail) or ''))
    end
end

local function tempdir()
    local dir = vim.fn.tempname()
    vim.fn.mkdir(dir, 'p')
    return dir
end

local function write_executable(path)
    vim.fn.mkdir(vim.fn.fnamemodify(path, ':h'), 'p')
    vim.fn.writefile({ '#!/bin/sh', 'exit 0' }, path)
    vim.fn.setfperm(path, 'rwxr-xr-x')
end

-- ---------------------------------------------------------------------------
-- 1. Spec structure (load the real file)
-- ---------------------------------------------------------------------------
local real_env = vim.env.GAZM_DEV_ROOT
vim.env.GAZM_DEV_ROOT = nil
local spec = dofile(spec_file)
vim.env.GAZM_DEV_ROOT = real_env

check('spec.main == "gazm"', spec.main == 'gazm', vim.inspect(spec.main))
check('spec.url == gazliddon/gazm.nvim', spec.url == 'https://github.com/gazliddon/gazm.nvim', vim.inspect(spec.url))
check('spec has dependencies', type(spec.dependencies) == 'table' and #spec.dependencies > 0)
check('spec.opts.lsp.command is a string', type(spec.opts.lsp.command) == 'string', vim.inspect(spec.opts.lsp.command))

-- ---------------------------------------------------------------------------
-- 2. LSP command auto-detection
-- ---------------------------------------------------------------------------
-- 2a. dev build present: GAZM_DEV_ROOT points at a fake checkout with
--     target/release/gazm + target/debug/gazm. release must win.
local fake = tempdir()
write_executable(fake .. '/target/release/gazm')
write_executable(fake .. '/target/debug/gazm')
vim.env.GAZM_DEV_ROOT = fake
local spec_release = dofile(spec_file)
check(
    'command prefers target/release/gazm',
    spec_release.opts.lsp.command == fake .. '/target/release/gazm',
    spec_release.opts.lsp.command
)

-- 2b. only debug build present -> debug wins
vim.fn.delete(fake .. '/target/release/gazm')
local spec_debug = dofile(spec_file)
check(
    'command falls back to target/debug/gazm',
    spec_debug.opts.lsp.command == fake .. '/target/debug/gazm',
    spec_debug.opts.lsp.command
)

-- 2c. no dev build -> `gazm` from PATH
vim.fn.delete(fake .. '/target/debug/gazm')
vim.fn.delete(fake .. '/target')
vim.env.GAZM_DEV_ROOT = fake
local spec_none = dofile(spec_file)
check('command falls back to "gazm" on PATH', spec_none.opts.lsp.command == 'gazm', spec_none.opts.lsp.command)

-- 2d. real environment: the command must be executable or a PATH name
vim.env.GAZM_DEV_ROOT = real_env
local spec_real = dofile(spec_file)
local real_cmd = spec_real.opts.lsp.command
check(
    'real environment command is usable',
    vim.fn.executable(real_cmd) == 1,
    real_cmd
)

vim.env.GAZM_DEV_ROOT = nil

-- ---------------------------------------------------------------------------
-- 3. lazy dev-mode resolution (uses the real dev config values)
-- ---------------------------------------------------------------------------
local Spec = require('lazy.core.plugin').Spec
local Config = require('lazy.core.config')

local function resolve_gazm(dev_dir_exists)
    local tmp = tempdir()
    local devroot = tmp .. '/dev'
    vim.fn.mkdir(devroot, 'p')
    if dev_dir_exists then
        vim.fn.mkdir(devroot .. '/gazm-plugin', 'p')
    end
    Config.options = vim.tbl_deep_extend('force', Config.defaults, {
        dev = {
            patterns = { 'gazliddon' },
            fallback = true,
            path = function(plugin)
                if plugin.name == 'gazm.nvim' then
                    return devroot .. '/gazm-plugin'
                end
                return devroot .. '/' .. plugin.name
            end,
        },
        root = tmp .. '/root',
    })
    local s = Spec.new()
    s:parse({
        {
            url = 'https://github.com/gazliddon/gazm.nvim',
            main = 'gazm',
        },
    })
    return s.plugins['gazm.nvim'] -- triggers rebuild
end

local with_dev = resolve_gazm(true)
check(
    'dev mode: local checkout used when present',
    with_dev.dev == true and with_dev.dir:match('dev/gazm%-plugin$') ~= nil,
    vim.inspect({ dev = with_dev.dev, dir = with_dev.dir })
)

local without_dev = resolve_gazm(false)
check(
    'dev mode: git fallback when checkout missing',
    without_dev.dev == false and without_dev.dir:match('root/gazm%.nvim$') ~= nil,
    vim.inspect({ dev = without_dev.dev, dir = without_dev.dir })
)

-- ---------------------------------------------------------------------------
-- 4. The real gaz/lazy.lua carries the dev block
-- ---------------------------------------------------------------------------
local lazy_src = table.concat(vim.fn.readfile(lazy_file), '\n')
check('gaz/lazy.lua has dev.patterns = {"gazliddon"}', lazy_src:find('patterns%s*=%s*{%s*["\']gazliddon', 1) ~= nil)
check('gaz/lazy.lua has dev.fallback = true', lazy_src:find('fallback%s*=%s*true', 1) ~= nil)
check('gaz/lazy.lua dev.path maps gazm.nvim', lazy_src:find('gazm%.nvim', 1) ~= nil)
check('gaz/lazy.lua dev.path maps to gazm-plugin', lazy_src:find('gazm%-plugin', 1) ~= nil)

print(string.format('\n%d passed, %d failed', pass, fail))
vim.cmd((fail == 0 and 'cquit' or 'cquit 1'))
