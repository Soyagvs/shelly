# Shelly command status capture

Feature: let scripts capture stdout, stderr, and exit code from system commands.

## Approved syntax

```shelly
command "cargo test" -> $out, $err, $code
```

## Semantics

- Existing syntax remains:
  - `command "..."`
  - `command "..." -> $stdout`
- New three-destination capture stores:
  - first destination: stdout
  - second destination: stderr
  - third destination: numeric exit code
- With three destinations, non-zero exit codes do not automatically fail the Shelly script; the script handles `$code`.
- With streaming or one-destination capture, existing failure behavior remains.
- Destinations must be mutable variables, not constants.
- Stdout and stderr captures trim trailing `\r`/`\n` consistently with one-destination stdout capture.
- If Rust `std::process::ExitStatus::code()` does not provide a code, Shelly stores `1`.

## Tasks

- [x] Explore current command execution and checker validation.
- [x] Implement three-destination command capture.
- [x] Add checker validation and clear errors.
- [x] Update docs/examples/tests.
- [x] Verify formatting and tests.

## Evidence

- `cargo fmt --check` passed.
- `cargo test` passed: 139 tests.
- Added runtime coverage for successful triple capture, non-zero triple capture continuing, one-destination non-zero failure preservation, and invalid destination forms/counts.
- Added checker coverage for triple capture and invalid destination counts/forms.
