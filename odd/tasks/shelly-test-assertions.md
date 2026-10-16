# Shelly test assertions

Feature: add assertions intended for `*_test.shelly` scripts.

## Decisions

- Add `assertEqual`, `assertTrue`, `assertFalse`, and `assertContains`.
- Assertions fail immediately with actionable expected/actual diagnostics.
- They are intended for tests, not recoverable application control flow.
- Existing `if` and `try/catch` remain the tools for normal program logic and recoverable errors.

## Tasks

- [x] Map expression/predicate handling and add RED assertion tests.
- [x] Implement assertion commands and diagnostics.
- [x] Document test assertion usage.
- [x] Verify focused and full suites.

## Evidence

- GREEN: focused assertion tests pass; failure coverage is split because assertions stop execution immediately.
- Documentation: updated `README.md` and `docs/project-structure.md` with assertion examples and test-only guidance.
- Final verification passed: `cargo fmt --check`; focused assertion tests (5/5); full suite (179/179). Runtime malformed assertions report arity errors without panicking.
- No commit was created; committing requires explicit user authorization.
