# Shelly parenthesized conditions

Feature: support grouping in `if`, `elif`, and `while` conditions.

## Decisions

- Parentheses are supported only in condition expressions.
- Supported condition sites: `if`, `elif`, and `while`.
- Parentheses can override logical precedence and can be nested.
- Existing precedence remains: `not`, then `and`, then `or`.
- `not (condition)` is valid.
- Predicates and comparisons work inside grouped conditions.
- Normal command syntax does not gain parentheses.
- Numeric expression parentheses remain supported where already documented.

## Tasks

- [x] Implement parenthesis-aware condition evaluation.
- [x] Implement parenthesis-aware checker validation.
- [x] Add tests for grouping, nested grouping, `not (...)`, predicates, `elif`, `while`, and malformed groups.
- [x] Update documentation.
- [x] Run verification.

## Evidence

- Added grouped condition support for `if`, `elif`, and `while`.
- Added nested groups and `not (condition)` support.
- Preserved precedence: `not`, then `and`, then `or`.
- Added checker validation for valid/malformed grouped conditions.
- Updated docs:
  - `docs/conditionals.md`
  - `docs/predicates.md`
  - `docs/while.md`
  - `docs/language.md`
  - `README.md`
- Verification passed:
  - `cargo fmt --check`
  - `cargo test parenthesized`
  - `cargo test grouped`
  - `cargo test` with 105 tests passed
  - manual smoke for grouped `if` and `while` passed
  - README links every `docs/*.md` file
- Native Gentle review inspect remained blocked by `package-local-binary-missing`.
