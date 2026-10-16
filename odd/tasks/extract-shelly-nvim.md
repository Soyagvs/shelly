# Extract Shelly Neovim plugin

Feature: publish the Neovim runtime as the standalone public `Soyagvs/shelly-nvim` plugin under GPL-3.0-or-later, then update Shelly documentation to point users to the installable plugin.

## Decisions

- Preserve the functional Neovim runtime layout: filetype detection, filetype settings, syntax, automatic LSP startup, and optional icon helpers.
- Do not copy SVG or font assets; they are not required for syntax or LSP functionality and their current paths are repository-relative.
- Use `shelly-lsp` from the user's `PATH`; `shelly update` remains responsible for updating the CLI and colocated LSP binary.
- Make the standalone README beginner-oriented, with a one-command Lazy.nvim path and a generic manual installation path.
- Keep licensing GPL-3.0-or-later by copying the existing `LICENSE` file.

## Tasks

- [x] Create `Soyagvs/shelly-nvim` and migrate the functional Neovim runtime.
- [x] Write repository-independent Neovim installation and troubleshooting documentation.
- [x] Remove the embedded runtime and update Shelly documentation links and tests.
- [x] Verify plugin structure and Shelly regression checks.
- [x] Commit, push, and publish the extracted plugin and the Shelly documentation update.

## Evidence

- Created `https://github.com/Soyagvs/shelly-nvim` as a public repository.
- Migrated the six runtime files and GPL license; all migrated runtime files byte-match the former embedded implementation.
- Excluded SVG and font assets because they are not needed by the plugin.
- Wrote standalone README with copy-paste Lazy.nvim and native Neovim-package installation, verification, troubleshooting, optional icons, and `shelly-lsp` update guidance.
- Updated the active Shelly documentation links; removed the embedded runtime and root-repository test coupled to it. `git diff --check` passed before the removals.
- Shelly verification passed: `cargo test` (238 tests), `git diff --check`, active-doc link checks, and no remaining live embedded-runtime paths.
- Plugin verification passed: runtime layout and GPL license checks, documentation checks, `git diff --check`, and a headless Neovim filetype test. The expected warning appeared because `shelly-lsp` was unavailable to the verifier.
- Published standalone plugin: <https://github.com/Soyagvs/shelly-nvim> at `3a5b5843b4299a222bb251ea9b8c2cceffb2ce42` (`feat: add Shelly Neovim runtime`).
- Merged and pushed Shelly documentation extraction at `ec55fed151a0a71859f034ccb22e687fb045a640` (`docs(neovim): link standalone plugin`).
