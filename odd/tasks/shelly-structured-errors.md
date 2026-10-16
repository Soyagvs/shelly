# Shelly structured errors

Feature: replace string-only caught runtime errors with structured error dictionaries.

## Decisions

- `try -> $error` binds a dictionary rather than a string.
- Every caught error contains `kind` and `message`.
- Process errors additionally expose `program`, `args`, `code`, `stdout`, and `stderr` when available.
- Existing string-only capture is intentionally replaced; Shelly has no external users yet.
- Control signals (`exit`, `break`, `continue`, and function `return`) remain uncaught.

## Tasks

- [x] Add failing integration tests that define the structured error contract.
- [x] Implement structured runtime errors for failed processes and ordinary runtime failures.
- [x] Update error-handling documentation and language references.
- [x] Correct formatting and nested-error documentation examples.
- [x] Run focused and full verification; record evidence.

## Evidence

- RED: `cargo test --test cli_smoke try` ran 5 tests; the 2 structured-error contract tests failed as intended because `$error` is still a scalar string (`cannot access key on non-dictionary value $error`).
- GREEN: `cargo test --test cli_smoke try` passed all 5 focused tests after runtime implementation.
- Documentation: updated `docs/errors.md` and `docs/commands.md` for structured caught-error fields and process failures.
- Verification attempt: focused tests passed (5/5) and full suite passed (156/156), but `cargo fmt --check` failed in `src/main.rs` and `tests/cli_smoke.rs`. Nested error examples in `docs/errors.md` also still interpolate dictionaries instead of `.message`.
- Correction: formatted Rust sources and corrected nested error examples to use `.message`; `cargo fmt --check` now passes.
- Final verification passed:
  - `cargo fmt --check`
  - `cargo test --test cli_smoke try_` (5/5)
  - `cargo test` (156/156)
- No verifier findings.
- Native review was unavailable because the package-local Gentle AI binary is missing; assessment treated the candidate as unassessable, and the required independent verifier passed.
- No commit was created; committing requires explicit user authorization.
