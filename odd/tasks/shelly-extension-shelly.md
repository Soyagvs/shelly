# Shelly extension migration

Feature: make `.shelly` the official script extension and remove `.ly` usage from active docs, examples, tests, and editor support.

## Goals

- Rename repository examples from `.ly` to `.shelly`.
- Update CLI help from `.ly` to `.shelly`.
- Update tests to use `.shelly` paths and references.
- Update README/docs/editor docs from `.ly` to `.shelly`.
- Update Neovim filetype/icon support from `.ly` to `.shelly`.
- Keep language runtime behavior unchanged apart from messaging and examples.
- Commit and push after verification.

## Tasks

- [x] Add failing/updated tests and rename examples to `.shelly`.
- [x] Update CLI/help/docs/editor integration.
- [x] Verify no active `.ly` references remain outside none.
- [x] Run full checks and example smoke test.
- [ ] Commit and push.

## Evidence

- Renamed `example-01.ly` to `example-01.shelly`.
- Renamed all `examples/*.ly` files to `examples/*.shelly`.
- Updated imports/includes to `.shelly`.
- Updated CLI help and tests to use `.shelly`.
- Updated Neovim detection/icon mapping to `.shelly`.
- Text search found no remaining active `.ly` references or files in README, docs, editors, src, tests, examples, and root showcase.
- `cargo fmt --check` PASS.
- `cargo test` PASS: 74 tests.
- Markdown local link check PASS: 16 markdown files checked.
- `printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly` PASS; final output included `Status: count is high enough`.
- `nvim --headless --cmd 'set runtimepath^=./editors/nvim' --cmd 'filetype on' example-01.shelly '+set filetype?' '+quit'` PASS: `filetype=shelly`.
