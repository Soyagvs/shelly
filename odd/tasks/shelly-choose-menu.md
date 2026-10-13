# Shelly choose menu command

Feature: add an interactive menu helper for simple choices.

## Decisions

- Add `choose "Question" ["A", "B"] -> $choice`.
- `choose` prints the question, then automatically numbers options from `1`.
- User input can be either the number or the exact option text.
- The destination receives the selected option value, not the index.
- Invalid input reprompts until a valid choice is entered.
- Empty option lists are errors.
- `choose` is for interactive menus, not command-line argument parsing.

## Tasks

- [x] Implement choose runtime behavior.
- [x] Add checker validation and CLI tests.
- [x] Document choose with examples.
- [x] Verify formatting, tests, and docs links.

## Evidence

- Added runtime support for `choose "Question" ["A", "B"] -> $choice`.
- Added checker validation for valid syntax, empty literal option lists, and invalid destinations.
- Added tests for:
  - choosing by number
  - choosing by exact text
  - invalid input reprompt
  - empty option list failure
  - check validation
- Added docs:
  - `docs/choose.md`
  - README docs table link
  - `docs/language.md` reference link
  - `docs/commands.md` reference note
- Verification passed:
  - `cargo fmt --check`
  - `cargo test choose` with 6 choose tests passed
  - `cargo test` with 125 tests passed
  - README local docs-link check passed
  - manual smoke with stdin `2` selected `B`
- Native Gentle review inspect remained blocked by `package-local-binary-missing`.
