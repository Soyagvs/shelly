# Shelly for Neovim

Local Neovim support for Shelly `.shelly` files.

## What this adds

- `.shelly` filetype detection as `shelly`.
- Basic syntax highlighting.
- Shelly comment settings for `# comment #`.
- Optional black file icon mapping for common Neovim icon plugins.

## Important icon note

Terminal Neovim cannot normally render an SVG file as the file explorer icon. Plugins such as `nvim-web-devicons` and `mini.icons` show **Nerd Font glyphs** with colors.

This package converts `../../shelly.svg` into `../../fonts/ShellyIcons.ttf` and maps `.shelly` to private-use glyph `U+E000` for Neovim. Install the font locally before expecting the exact logo to render.

## Try it without installing

From the repository root, open Neovim with this runtime path:

```sh
nvim --cmd 'set runtimepath^=./editors/nvim' example-01.shelly
```

Inside Neovim, check:

```vim
:filetype on
:set filetype?
:syntax on
```

Expected filetype:

```txt
filetype=shelly
```

Shelly uses `.shelly` to avoid extension conflicts in Neovim and other editor tooling.

## Install locally

Copy the runtime files into your Neovim config:

```sh
mkdir -p ~/.config/nvim/ftdetect ~/.config/nvim/ftplugin ~/.config/nvim/syntax ~/.config/nvim/lua
cp editors/nvim/ftdetect/shelly.lua ~/.config/nvim/ftdetect/
cp editors/nvim/ftplugin/shelly.vim ~/.config/nvim/ftplugin/
cp editors/nvim/syntax/shelly.vim ~/.config/nvim/syntax/
cp editors/nvim/lua/shelly-icons.lua ~/.config/nvim/lua/
```

## File icon with `nvim-web-devicons`

If your file explorer uses [`nvim-web-devicons`](https://github.com/nvim-tree/nvim-web-devicons), add this after the plugin loads:

```lua
require("shelly-icons").setup_web_devicons()
```

Or customize the glyph/color:

```lua
require("shelly-icons").setup_web_devicons({
  icon = "",
  color = "#000000",
  cterm_color = "16",
  name = "Shelly",
})
```

## File icon with `mini.icons`

If you use [`mini.icons`](https://github.com/echasnovski/mini.icons), add:

```lua
require("shelly-icons").setup_mini_icons()
```

## Lazy.nvim example

```lua
{
  dir = "/home/soyagvs/proyectos/personal/shelly/editors/nvim",
  name = "shelly.nvim",
  lazy = false,
  config = function()
    require("shelly-icons").setup_web_devicons()
  end,
}
```

Adjust the `dir` path if the repository lives somewhere else.
