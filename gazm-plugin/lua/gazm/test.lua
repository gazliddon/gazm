vim.api.nvim_create_user_command("Test", function ()
    package.loaded["gazm.text"] = nil
    require("gazm.text").hl()
end, {})
