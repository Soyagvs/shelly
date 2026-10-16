# Shelly self-update

Feature: secure, user-invoked `shelly update` for released Shelly binaries.

## Decisions

- Updates are manual: no background check or automatic mutation.
- Update the colocated `shelly` and `shelly-lsp` executables together.
- Download only official GitHub release assets, validate the matching SHA-256 checksum before extraction, and fail without replacement on invalid input.
- Support the published targets: Linux x86_64, macOS x86_64/aarch64, and Windows x86_64.

## Tasks

- [x] Add testable release asset, version, platform, and checksum primitives.
- [x] Implement the `shelly update` command and safe executable replacement.
- [x] Document updater behavior, requirements, and failure modes.
- [x] Run focused and full verification.

## Evidence

- `cargo fmt --check`: passed.
- `cargo test`: passed — 224 tests.
- `cargo check --locked`: passed.
- `git diff --check`: passed.
- Tests cover checksum mismatch, strict versioned asset selection, unsafe archive entries, paired rollback, and Windows hand-off argument validation.
- ZIP and tar archives accept only regular root-level Shelly binaries. Release tags must be `vMAJOR.MINOR.PATCH` or `MAJOR.MINOR.PATCH`.
- Windows waits for the original process (bounded to 30 seconds) before the paired transactional replacement; its platform-specific execution was not run on this non-Windows host.
- No commit created; awaiting explicit user authorization.
