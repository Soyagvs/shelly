# Shelly regex predicates

Feature: validate text with standard Rust regular expressions through a readable predicate.

## Decisions

- Add `matches <value> /pattern/` as a boolean predicate usable in conditions and predicate capture.
- Use the Rust `regex` crate, preserving its familiar syntax and linear-time matching behavior.
- A regex literal is delimited by `/`; escape a literal slash as `\/`.
- Matching is substring-based unless the author uses `^` and `$` anchors.
- Invalid syntax reports an actionable error instead of aborting the interpreter.

## Tasks

- [x] Map existing predicate parsing, checker validation, and CLI test conventions.
  - Evidence: `exists`/`contains` flow through runtime, capture, and static validation in `src/lib.rs`; condition scanners must become regex-literal-aware.
- [x] Add the `matches` predicate, regex-literal parsing, and focused tests (RED then GREEN).
- [x] Document syntax, anchors, character classes, ranges, symbols, and limitations.
- [x] Run formatting and focused/full verification.
- [x] Map the VS Code extension, Neovim plugin, and LSP integration points for regex literals.
  - Evidence: LSP lexer treats `/` as generic punctuation (`src/lsp/syntax.rs`); VS Code and Neovim grammar operator rules also need a preceding regex-literal rule; both editor clients pass LSP traffic through unchanged.
- [x] Update editor integrations and LSP coverage for `matches` regex literals.
  - LSP evidence: added complete-literal scanning after `matches` in `src/lsp/syntax.rs`; `cargo test --test lsp_syntax` passed 6 tests and `cargo test --test lsp_features` passed 8 tests. RED was observed before scanning was implemented.
  - VS Code evidence: added the ordered TextMate regex-literal rule and `matches` builtin highlighting; `node --test test/extension.test.js` passed 12 tests after an observed two-failure RED state.
  - Neovim evidence: added the `matches` predicate and ordered regex literal match in `syntax/shelly.vim`; `vim --clean -Nu NONE -n -es +'source syntax/shelly.vim' +q` passed. Test-first was not applicable because the plugin has no syntax test harness.
- [x] Run repository-specific verification across Shelly, VS Code, and Neovim.
  - Independent verification passed: focused LSP (6 tests), VS Code grammar (12 tests), and Vim syntax loading. `git diff --check` passed in all three repositories. Live editor rendering was not automated.

## Evidence

- RED: `cargo test --test cli_smoke regex_literals_do_not_confuse_condition_scanning -- --exact` failed before the scanner fix with `unexpected text after regex literal`.
- GREEN: the same focused test passed after wiring `mask_regex_literals` into keyword/operator/group scanning.
- Added CLI coverage for direct matching, `not`, boolean composition, capture, anchored `^[a-z]+\\d+$`, escaped slash, valid `check`, and invalid regex/check.
- Inline flag documentation uses idiomatic `/(?i)^[a-z]+$/`.

- Independent verification passed: `cargo fmt --check`, `cargo test --test cli_smoke regex` (2 tests), and `cargo test` (all tests, including 207 CLI smoke tests).
- `git diff --check` passed; the pre-existing untracked `.codegraph/` directory was not modified.
- Native review was unavailable because the package-local Gentle AI binary is missing; independent verification was completed instead.
