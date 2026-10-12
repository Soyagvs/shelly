# Shelly Neovim icon glyph polish

Feature: replace the terminal-like Shelly Neovim glyph with a cleaner language mark.

## Goals

- Keep Neovim compatible with terminal icon rendering.
- Avoid terminal-looking glyphs for `.shelly` files.
- Use a stable purple glyph that works even when SVG rendering is unavailable.
- Update both project defaults and user Neovim config.
- Verify icon resolution.
- Commit and push project changes.

## Tasks

- [x] Change Shelly Neovim icon glyph.
- [x] Verify user Neovim resolves the new icon.
- [ ] Commit and push.

## Evidence

- Changed project default Shelly Neovim glyph from terminal-like `󰞷` to `S`.
- Changed user config `~/.config/nvim/lua/plugins/shelly.lua` to use `S`.
- User Neovim check PASS: `filetype=shelly`, `devicon=S color=#7c3aed`.
- `cargo fmt --check` PASS.
- `cargo test` PASS: 74 tests.
