# Shelly conditionals

Feature: add `if` / `elif` / `else` control flow.

## Decisions

- Conditionals start with `if` and end with `end`.
- Optional middle branches use `elif`.
- Optional fallback branch uses `else`.
- Syntax uses no parentheses, no `then`, and no braces.
- Initial comparison operators: `==`, `!=`, `>`, `>=`, `<`, `<=`.
- Boolean direct conditions are allowed: `if $enabled`.
- Truthiness rules:
  - `true` is true.
  - `false` is false.
  - empty string is false.
  - number `0` is false.
  - everything else is true.
- `if` blocks do not create a new scope; assignments inside a taken branch affect the current scope.
- Initial implementation excludes `and`, `or`, `not`, parentheses, and complex expressions.

## Tasks

- [x] Add failing tests for `if`, `elif`, `else`, comparisons, truthiness, and scope behavior.
- [x] Implement conditional parsing and execution.
- [x] Add negative tests for malformed conditionals and unsupported expressions.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed before implementation with `unknown command or invalid syntax: if $enabled` and `unknown command or invalid syntax: if true`.
- GREEN: `cargo test` passed after implementation: 17 passed.
- Format: `cargo fmt --check` passed.
- Manual: `cargo run --quiet -- examples/conditionals.ly` printed `enabled branch` and `interpolation works`.
