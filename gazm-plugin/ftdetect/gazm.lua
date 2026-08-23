-- ftdetect/gazm.lua
-- Gazm is only associated via the `.gazm` extension. Buffer-local options
-- (commentstring etc.) live in ftplugin/gazm.lua.
vim.filetype.add({
    extension = {
        gazm = 'gazm',
    },
})
