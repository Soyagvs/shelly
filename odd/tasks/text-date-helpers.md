# Add text and date formatting helpers

Feature: add substring extraction, left/right padding, and UTC ISO date formatting to Shelly standard helpers.

## Decisions

- `substring TEXT START LENGTH -> $value` uses one-based character indexing and explicit length, matching Shelly list indexing.
- `padLeft TEXT WIDTH FILL -> $value` and `padRight TEXT WIDTH FILL -> $value` pad only when text is shorter than width; fill must be one character.
- `formatDate ISO_UTC PATTERN -> $value` parses UTC ISO timestamps such as `now` returns and supports explicit patterns `YYYY`, `MM`, `DD`, `HH`, `mm`, and `ss`.
- Invalid ranges, invalid ISO UTC input, unsupported pattern tokens, non-positive substring positions, and invalid fill values fail with actionable errors.

## Tasks

- [x] Map helper implementation and add failing behavior tests.
- [x] Implement text/date helpers and documentation.
- [x] Run focused and full verification.
- [ ] Commit the helper feature.

## Evidence

- RED observed: `cargo test --test cli_smoke text_and_date_helpers` ran one matching test; it failed because `substring` is unknown.
- Mapped runtime dispatch, helper registry, check-time validation, existing UTC helpers, integration test conventions, and documentation surface before implementation.
- GREEN observed: helper runtime, check-time validation, strict UTC ISO formatting, and documentation are implemented; focused tests pass 2/2 after correcting the test's expected newline.
- Independent verification passed: focused tests (2/2), full test suite including 203 CLI smoke tests, and diff check.
