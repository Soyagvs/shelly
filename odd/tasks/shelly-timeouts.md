# Shelly timeouts

Feature: add process and block timeouts.

## Decisions

- `command ... timeout SECONDS` limits one direct process.
- `timeout SECONDS ... end` gives the complete nested block one total deadline.
- Block deadlines are inherited by functions and loops.
- A timeout raises structured error kind `timeout`.
- Timeout values are positive seconds and may include fractions; automatic timeouts are out of scope.

## Tasks

- [x] Map process execution, blocks, control flow, and add RED tests.
- [x] Implement direct process and inherited block deadlines.
- [x] Document timeout syntax and semantics.
- [x] Verify focused and full suites.

## Evidence

- Focused timeout integration tests: 4 passed.
- `cargo fmt --check`: passed.
- `cargo test`: 201 passed.
- `git diff --check`: passed.
- No commit created; awaiting explicit user authorization.
