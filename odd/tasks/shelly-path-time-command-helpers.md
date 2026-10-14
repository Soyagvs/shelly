# Shelly path, time, and command helpers

Feature: add migration-friendly standard helpers for paths, timestamps, and command discovery.

## Scope

- Add `pathJoin`, `dirname`, `basename`, optional basename extension stripping, `timestamp`, `commandExists`, and `which`.
- Add focused tests for the new helpers.
- Document the new helpers for Bash-to-Shelly migration use cases.

## Tasks

- [x] Explore helper implementation and tests.
- [x] Add path/time/command helpers with tests.
- [x] Update docs for new helpers.
- [x] Run focused verification.

## Evidence

- Added `pathJoin`, optional extension stripping for `basename PATH EXT`, `timestamp`, `commandExists`, and `which` helpers.
- Kept existing `dirname`, `basename`, `extension`, `withoutExtension`, `now`, and `today` behavior while extending docs and tests.
- Updated `docs/helpers.md` with path, time, and command-discovery examples.
- Focused verification PASS: `cargo fmt --check`; `cargo test standard_helper_commands_cover_text_lists_types_paths_time_and_random`; `cargo test standard_helpers_check_valid_syntax_and_reject_representative_errors`; `cargo run --quiet -- examples/conditionals.shelly`.
