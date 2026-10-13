# Shelly log blocks

Feature: add structured `log` blocks for script execution traces.

## Decisions

- Add `log "identifier"` / `end` blocks.
- Use one log file per identifier per day to avoid one large daily file:
  - `logs/<identifier>-YYYY-MM-DD.log`
- Resolve `logs/` relative to the current Shelly script directory.
- The log identifier is required and is sanitized for filenames.
- `log` does not silence normal script output.
- `shellout` still prints normally.
- The log records block lifecycle:
  - `ENTER`
  - selected Shelly actions inside the block
  - `EXIT ok`
  - `ERROR <message>` when an ordinary error occurs
- `log` does not capture full stdout/stderr from external `command` output in the first version; it records the command statement and whether the block exits with error.
- Control signals keep their normal behavior.

## Tasks

- [x] Implement `log "identifier"` block parsing and execution.
- [x] Record structured log lines with timestamps.
- [x] Add checker validation and CLI tests.
- [x] Document log blocks.
- [x] Verify formatting and tests.

## Evidence

- Added runtime support for:
  - `log "identifier"`
  - body statements
  - `end`
- Added script-relative log storage:
  - `logs/<sanitized-identifier>-YYYY-MM-DD.log`
- Added log entries for lifecycle and selected actions:
  - `--- identifier ---`
  - `ENTER`
  - `shellout: ...`
  - `command: ...`
  - file command markers
  - nested log markers
  - `EXIT ok`
  - `ERROR ...`
  - `EXIT control ...` where practical
- Added docs:
  - `docs/logging.md`
  - README docs table link
  - `docs/language.md` reference link
  - `docs/commands.md` reference note
- Verification passed:
  - `cargo fmt --check`
  - `cargo test log_block` with 3 targeted tests passed
  - `cargo test` with 117 tests passed
  - README docs-link check passed
  - manual smoke preserved stdout and created `logs/build-YYYY-MM-DD.log` with `ENTER`, `shellout`, `command`, and `EXIT ok`
- Native Gentle review inspect remained blocked by `package-local-binary-missing`.
