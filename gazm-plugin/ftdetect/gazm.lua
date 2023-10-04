-- ftdetect/gazm.lua
-- executed whenever a file of type gazm is loaded

-- Guard against reoload
if vim.g.did_load_gazm then
    return
end

-- Say we've been loaded
vim.g.did_load_gazm = true

-- Create a gazm au group

local id = vim.api.nvim_create_augroup("gazmgroup", {
    clear = true
})

-- Create an autocommand where any files of type gazm
-- that are loaded will set the filetype to gazm
vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter" }, {
    group = id,
    pattern = { "*.gazm" },
    callback = function()
        if vim.bo.filetype ~= 'gazm' then
            vim.bo.filetype='gazm'
            -- @TODO errorformat
            -- @TODO makeprg
        end
    end
})
