# Shelly quality pass

Feature: mature existing Shelly behavior without adding new language features.

## Scope

- Improve existing error messages where useful.
- Add clearer tutorial-style documentation.
- Change `ask` from comma syntax to arrow syntax.
- Refactor the Rust implementation out of one large `src/main.rs` into focused modules.

## Decisions

- `ask` official syntax becomes `ask "Question?" -> $answer`.
- Old comma-style `ask` should be rejected with guidance toward arrow syntax.
- Mental model: `->` means command result/binding into a variable; commas remain normal argument separators for calls and list commands where appropriate.
- Documentation should explain existing features in a more guided order, not only as a rule dump.
- Refactor must preserve behavior and tests.
- This task should not add new Shelly features beyond the `ask` syntax migration.

## Tasks

- [x] Add failing tests for `ask "..." -> $var` and old comma-form rejection.
- [x] Implement ask arrow syntax.
- [x] Improve targeted diagnostics for existing behavior without changing semantics.
- [x] Add tutorial-style documentation for existing language features.
- [x] Refactor `src/main.rs` into focused modules.
- [x] Update examples and docs for ask arrow syntax.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed after adding tests with 62 passed / 3 failed. Failures covered arrow ask still rejected as comma-argument syntax, missing migration guidance for old comma ask, and README still using old comma ask syntax.
- GREEN: `cargo fmt --check && cargo test` passed. `cargo test` reported 65 passed / 0 failed.
- Sample run: `printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` passed and completed end-to-end, including `Do you want to continue? Great, continuing` and `Favorite color: blue`.
- Files changed: `src/main.rs`, `src/comments.rs`, `src/parser.rs`, `src/shell.rs`, `src/value.rs`, `tests/cli_smoke.rs`, `README.md`, `docs/language.md`, `examples/ask-if.ly`, `example-01.ly`, `odd/tasks/shelly-quality-pass.md`.
- Refactor module summary: `comments` owns comment stripping, `parser` owns lexical helpers/string and identifier parsing, `shell` owns stdin and system command helpers, `value` owns value/identifier types and value formatting/comparison helpers. `main.rs` retains CLI flow and interpreter control logic.
- Diagnostics changed: old comma `ask` reports migration guidance to `ask "Question?" -> $answer`; invalid ask destinations report variable-destination guidance; list out-of-range diagnostics include the target and length where list context is available.
- Deferred: no full error-system rewrite; interpreter control flow and numeric parser remain in `main.rs` to avoid a broad semantic rewrite during this quality pass.
