# Single-dollar constants

Feature: migrate Shelly constants from `$$NAME` to `$NAME`.

## Approved direction

This is an intentional breaking syntax change. Constants use one dollar sign and an all-uppercase identifier; variables use one dollar sign and an all-lowercase identifier.

```shelly
$APP = "Shelly"        # constant
$app_name = "shelly"   # variable
```

Mixed-case identifiers and the legacy `$$NAME` syntax are invalid.

## Tasks

- [x] Map all parser, runtime, test, documentation, and example surfaces using identifier prefixes.
- [x] Change identifier classification, validation, formatting, and diagnostics.
- [x] Migrate tests, documentation, and examples to the new syntax; add incompatibility coverage.
- [x] Run formatting, complete test suite, and documentation checks.

## Constraints

- No compatibility mode for `$$NAME`.
- No Git commit unless explicitly requested.

## Evidence

- Work-unit commit: `82c7f57 feat: simplify constant syntax`.
- RED observed: migrated syntax tests failed before the parser migration.
- `cargo fmt -- --check` passed.
- `cargo test --test cli_smoke` passed: 155 tests.
- `cargo test` passed: 155 tests.
- Product source, README, docs, examples, and tests contain no legacy `$$` except intentional rejection diagnostics and fixtures.
- README, docs, and examples contain no mixed-case variable identifiers; current docs explicitly require lowercase variables and uppercase constants.
- Historical `odd/tasks/` records retain their original `$$` examples as past-work evidence; they are not current language documentation.
