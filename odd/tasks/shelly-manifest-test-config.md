# Shelly manifest test configuration

Feature: make `Shelly.toml` configure the test runner.

## Decisions

- Read `[project]` keys `name`, `version`, and `entry`; validate their required string values.
- Read `[test]` keys `directory` and `pattern`.
- `shelly test` uses configured values, defaulting to `tests` and `*_test.shelly` when `[test]` is omitted.
- `shelly run` is out of scope.

## Tasks

- [x] Map manifest parsing and add RED tests.
- [x] Implement TOML manifest validation and configured test discovery.
- [x] Document manifest test configuration.
- [x] Verify focused and full suites.

## Evidence

- Design: use the `toml` crate with Serde structs; migrate init and existing test manifests to `[project]` shape.
- GREEN: focused CLI suite passed (183/183) after implementation.
- Documentation: updated `README.md` and `docs/project-structure.md` with manifest requirements and test settings.
- Final verification passed: `cargo fmt --check`; `cargo test --test cli_smoke` (183/183); `cargo test` (183/183).
- No commit was created; committing requires explicit user authorization.
