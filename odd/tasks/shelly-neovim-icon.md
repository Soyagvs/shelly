# Shelly Neovim icon and filetype support

Feature: let `.ly` files feel like a real language in Neovim first, before global/editor-wide distribution.

## Goals

- Keep the user-provided `shelly.svg` in the repository for future editor packages.
- Add local Neovim runtime files for `.ly` detection and basic highlighting.
- Add optional icon mappings for common Neovim icon plugins.
- Document the terminal limitation: Neovim file explorers usually display Nerd Font glyphs, not SVG images.
- Do not change Shelly language runtime behavior.
- Commit and push after verification.

## Files

- `shelly.svg`
- `editors/nvim/README.md`
- `editors/nvim/ftdetect/shelly.lua`
- `editors/nvim/ftplugin/shelly.vim`
- `editors/nvim/syntax/shelly.vim`
- `editors/nvim/lua/shelly-icons.lua`
- `README.md`

## Tasks

- [x] Add Neovim filetype detection and basic syntax support.
- [x] Add icon plugin helper/config for `.ly`.
- [x] Document how to test locally in Neovim.
- [x] Verify docs/links and Rust tests.
- [ ] Commit and push.

## Evidence

- The fetched `shelly.svg` is kept as the canonical visual asset.
- The stray `shelly.svg:Zone.Identifier` file was removed before versioning.
- Neovim limitation documented: terminal file explorers use Nerd Font glyphs, not SVG images.
- Neovim already maps `.ly` to LilyPond by default; Shelly's local runtime overrides `.ly` to `filetype=shelly` when loaded with `filetype on`.
- `cargo fmt --check` PASS.
- `cargo test` PASS: 74 tests.
- Markdown local link check PASS: 16 markdown files checked.
- `printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` PASS; final output included `Status: count is high enough`.
- `nvim --headless --cmd 'set runtimepath^=./editors/nvim' --cmd 'filetype on' example-01.ly '+set filetype?' '+quit'` PASS: `filetype=shelly`.
