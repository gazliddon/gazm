-- Guard against reoload
if vim.g.did_load_gazm then
    return
end

vim.g.did_load_gazm = true

local id = vim.api.nvim_create_augroup("gazmgroup", {
    clear = true
})

vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter" }, {
    group = id,
    pattern = { "*.gazm" },
    callback = function(_ev)
        vim.bo.filetype='gazm'
    end
})
