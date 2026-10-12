# Shelly Neovim user icon enablement

Feature: enable the Shelly `.shelly` file icon in the user's Neovim setup.

## Goals

- Add a local Lazy.nvim plugin spec that loads Shelly's Neovim runtime from this repository.
- Register `.shelly` filetype support.
- Register a purple Shelly icon through Neovim icon providers.
- Verify user Neovim reports `filetype=shelly` and resolves an icon for `.shelly` files.
- Commit and push project task evidence.

## Tasks

- [x] Add Neovim user plugin configuration.
- [x] Verify filetype and icon resolution.
- [x] Commit and push project task evidence.

## Evidence

- Added user config at `~/.config/nvim/lua/plugins/shelly.lua`.
- The config loads `/home/soyagvs/proyectos/personal/shelly/editors/nvim` as a local Lazy.nvim plugin.
- `.shelly` resolves to `filetype=shelly` in the user's real Neovim config.
- `nvim-web-devicons` resolves `example-01.shelly` to icon `󰞷` with color `#7c3aed`.
- Fixed Shelly syntax highlighting regex so Neovim no longer reports syntax regexp errors during startup.
- `cargo fmt --check` PASS.
- `cargo test` PASS: 74 tests.
- Markdown local link check PASS: 16 markdown files checked.
- `printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly` PASS; final output included `Status: count is high enough`.
- User Neovim icon check PASS: `filetype=shelly`, `devicon=󰞷 color=#7c3aed`.
