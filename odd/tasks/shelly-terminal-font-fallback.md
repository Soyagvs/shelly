# Shelly terminal font fallback

Feature: make the custom Shelly icon font render in the user's terminal instead of showing a purple square.

## Goals

- Stop using a high private-use codepoint that terminal font fallback may not render.
- Move Shelly glyph to safer private-use codepoint `U+E000`.
- Configure the user's terminal font fallback to include `ShellyIcons` before the Nerd Font.
- Keep Neovim icon mapping aligned with the generated font.
- Verify Neovim still resolves the glyph and project checks pass.
- Commit and push project changes.

## Tasks

- [x] Regenerate Shelly icon font at `U+E000`.
- [x] Update Neovim icon mappings.
- [x] Configure terminal font fallback.
- [x] Verify and commit/push.

## Evidence

- Regenerated `fonts/ShellyIcons.svg` and `fonts/ShellyIcons.ttf` with Shelly glyph at `U+E000`.
- Updated `scripts/generate-icon-font.mjs` to generate `U+E000`.
- Updated project Neovim defaults and user config to use glyph `U+E000`.
- Updated user WezTerm config at `~/.wezterm.lua` so `ShellyIcons` is first in `font_with_fallback`, before `IosevkaTerm Nerd Font`.
- `cargo fmt --check` PASS.
- `cargo test` PASS: 74 tests.
- Markdown local link check PASS: 16 markdown files checked.
- `printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly` PASS; final output included `Status: count is high enough`.
- User Neovim check PASS: `filetype=shelly`, `devicon=<U+E000> color=#7c3aed`.

## Note

A full terminal restart is required for WezTerm/font fallback changes to take effect visually.
