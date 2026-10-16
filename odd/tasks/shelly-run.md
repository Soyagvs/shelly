# Shelly project run

Feature: add `shelly run` to execute the configured project entrypoint.

## Decisions

- `shelly run` requires `Shelly.toml` in the current directory.
- It reads validated `[project].entry` and executes that script.
- Project arguments follow `--` and are forwarded unchanged.
- Direct `shelly script.shelly` execution remains supported.

## Tasks

- [x] Map dispatch and manifest reuse; add RED tests.
- [x] Implement run entrypoint and argument forwarding.
- [x] Document project run workflow.
- [x] Verify focused and full suites.

## Evidence

- RED: five focused `shelly run` tests fail because `run` is still treated as a script path.
- Final verification passed: `cargo fmt --check`; focused run tests (8/8); full suite (189/189).
- No commit was created; committing requires explicit user authorization.
