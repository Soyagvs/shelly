# Shelly custom Neovim icon font

Feature: convert the Shelly SVG logo into a font glyph so terminal Neovim icon providers can display the real project icon.

## Goals

- Generate a font from `shelly.svg`.
- Map `.shelly` files to that private-use glyph in Neovim icon providers.
- Install the generated font locally for the user when possible.
- Keep the SVG as source artwork.
- Verify Neovim resolves the glyph and project checks still pass.
- Commit and push project assets/config.

## Tasks

- [x] Generate Shelly icon font from SVG.
- [x] Install local font and refresh font cache.
- [x] Update Neovim mappings from fallback glyph to Shelly glyph.
- [x] Verify Neovim icon resolution and project checks.
- [ ] Commit and push.

## Evidence

- Generated `fonts/ShellyIcons.svg` and `fonts/ShellyIcons.ttf` from `shelly.svg`.
- The Shelly glyph is mapped to private-use codepoint `U+100000` to avoid common Nerd Font icon ranges.
- Installed the font locally at `~/.local/share/fonts/Shelly/ShellyIcons.ttf` and refreshed font cache.
- `fc-match ':charset=100000'` resolves to `ShellyIcons.ttf`.
- Updated project Neovim defaults and user config to use the `U+100000` glyph.
- User Neovim check resolves `.shelly` as `filetype=shelly` and `devicon=<U+100000> color=#7c3aed`.

- `cargo fmt --check` PASS.
- `cargo test` PASS: 74 tests.
- Markdown local link check PASS: 16 markdown files checked.
- `printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly` PASS; final output included `Status: count is high enough`.
- Final user Neovim check PASS: `filetype=shelly`, `devicon=<U+100000> color=#7c3aed`.
