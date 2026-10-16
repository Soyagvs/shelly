# Shelly test runner

Feature: add `shelly test` to discover and run project test scripts.

## Decisions

- `shelly test` runs from a project root that contains `Shelly.toml`.
- It recursively discovers `*_test.shelly` files under `tests/`.
- Each test runs in a separate Shelly process, isolating interpreter state.
- A test passes only when its process exits with code 0; any nonzero code or runtime failure fails it.
- The runner prints one `PASS` or `FAIL` result per test and a final passed/failed summary.
- The runner exits 0 only when every discovered test passes; it exits nonzero for test failures, missing project manifest, missing test directory, or no discovered tests.
- Native assertion helpers are out of scope; tests use existing Shelly control flow and `exit`.

## Tasks

- [x] Map CLI, subprocess, recursive discovery, and test conventions; add failing runner tests.
- [x] Implement discovery, isolated execution, reporting, and exit status.
- [x] Document `shelly test` workflow and test naming.
- [x] Run focused and full verification; record evidence.

## Evidence

- Design: discovery paths are normalized project-relative paths and sorted lexicographically before execution. The runner continues after failures so every discovered test is reported.
- RED: `cargo test --test cli_smoke test_ -- --nocapture` ran six runner contract tests; all failed as intended before implementation.
- GREEN: `cargo test --test cli_smoke test_` passed all 6 focused runner tests.
- Documentation: updated `README.md` and `docs/project-structure.md` with test naming, execution, isolation, summary, and current assertion scope.
- Final verification passed: `cargo fmt --check`; focused runner tests (6/6); full suite (172/172). No verifier findings.
- No commit was created; committing requires explicit user authorization.
