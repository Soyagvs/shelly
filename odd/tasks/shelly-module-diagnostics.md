# Shelly module diagnostics

Feature: improve local import diagnostics and validate import graphs with `shelly check`.

## Decisions

- Detect missing imports, import chains, and circular imports clearly.
- `shelly check` validates imported modules without executing them.
- No new module syntax or external package system.

## Tasks

- [x] Map import runtime/checker and add RED tests.
- [x] Implement import graph validation and diagnostics.
- [x] Document module diagnostics.
- [x] Verify focused and full suites.

## Evidence

- Final verification passed: `cargo fmt --check`; `cargo test check` (33/33); full suite (197/197).
- No commit was created; committing requires explicit user authorization.
