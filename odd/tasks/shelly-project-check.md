# Shelly project check

Feature: allow `shelly check` with no script path to validate the configured project entrypoint.

## Decisions

- `shelly check` loads current-directory `Shelly.toml` and validates `[project].entry` without executing it.
- `shelly check path.shelly` remains unchanged for standalone scripts.

## Tasks

- [x] Map check dispatch and add RED tests.
- [x] Implement project-entry check.
- [x] Document check workflow.
- [x] Verify focused and full suites.

## Evidence

- Final verification passed: `cargo fmt --check`; focused project-check tests (28/28); full suite (193/193).
- No commit was created; committing requires explicit user authorization.
