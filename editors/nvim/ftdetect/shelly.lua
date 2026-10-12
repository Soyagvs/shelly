-- Neovim already maps `.ly` to LilyPond by default.
-- Shelly projects intentionally override that extension locally.
vim.api.nvim_create_autocmd({ "BufRead", "BufNewFile" }, {
  pattern = "*.ly",
  callback = function()
    vim.bo.filetype = "shelly"
  end,
})
