# Shelly safe command syntax

Feature: make `command` execute a program and explicit arguments without shell parsing, and provide `shell` for intentional shell-string execution.

## Decisions

- `command git status --short` executes `git` directly with `status` and `--short` as arguments.
- Values such as `$url` remain one process argument and are never re-parsed as shell code.
- `shell "..."` is the explicit escape hatch for shell syntax such as pipelines and redirections.
- This is an intentional breaking change: quoted string command syntax is removed because Shelly has no external users yet.
- Existing one- and three-destination output capture semantics are retained for `command`.

## Tasks

- [x] Map grammar and add failing integration tests for direct command and explicit shell behavior.
- [x] Implement parser and direct process execution for `command`, plus explicit `shell` execution.
- [x] Update command and language documentation with migration guidance.
- [x] Correct checker regression and migrate legacy test fixtures.
- [x] Run focused and full verification; record evidence.

## Evidence

- Grammar decision: `command <program> <args...>` uses whitespace-separated bare words, single-line quoted string literals (with interpolation), and variable references as separate argv values. Triple-backtick arguments are out of scope. `shell` retains the same one-/three-destination capture forms.
- RED: seven focused integration tests failed as intended against the legacy quoted shell-string command behavior.
- GREEN: all seven focused direct-command and explicit-shell contract tests pass after completing runtime/parser implementation.
- Documentation: updated `docs/commands.md`, `docs/language.md`, and `README.md` with direct command syntax, explicit shell behavior, capture forms, and migration guidance.
- Verification attempt failed: `cargo fmt --check` found formatting differences; `cargo test` had 25 failures. The checker incorrectly validates unrelated whitespace-containing statements, and existing tests still use the intentionally removed quoted `command` form.
- Correction: checker validation now targets command/shell statements only; legacy fixtures migrated to direct argv or explicit shell syntax; `cargo fmt --check` and `cargo test` (163 tests) pass.
- Final independent verification passed:
  - `cargo fmt --check`
  - seven focused direct-command and explicit-shell contract tests
  - `cargo test` (163/163)
- No verifier findings.
- Native review was unavailable because the package-local Gentle AI binary is missing; assessment treated the candidate as unassessable, and the required independent verifier passed.
- No commit was created; committing requires explicit user authorization.
