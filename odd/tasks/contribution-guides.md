# Add repository-specific contribution guides

Feature: give contributors clear, accurate, and independently usable CONTRIBUTING guides for Shelly, Shelly VS Code, and Shelly Neovim.

## Decisions

- Each repository owns its guide because setup, validation, and release procedures differ.
- Guides lead with a quick path, then provide scoped contribution, validation, and release information.
- Only verified repository commands are documented; manual checks are clearly labeled where no automated runner exists.

## Tasks

- [x] Write a tailored `CONTRIBUTING.md` for Shelly.
- [x] Write tailored `CONTRIBUTING.md` guides for Shelly VS Code and Shelly Neovim.
- [x] Verify links and documented commands, then commit and publish all three repositories.

## Evidence

- The audit found no existing contributor or governance guides in any repository.
- Shelly uses Rust CI; the VS Code extension uses npm packaging and tests; the Neovim plugin currently relies on manual headless/runtime validation.
- Shelly guide was validated with formatting, full tests, sample execution, and targeted CLI/LSP/release packaging tests.
- VS Code guide was validated with npm validation, build, and ten tests; Neovim guide was validated with a headless syntax load.
- Independent cross-repository review confirmed all guide links/anchors and documented commands; diff checks passed in all repositories.
- Published to all three `main` branches. Editor repositories rebased cleanly over concurrent remote badge updates before publication.
