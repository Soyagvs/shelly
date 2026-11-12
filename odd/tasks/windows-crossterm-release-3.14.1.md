# Windows Crossterm Release v3.14.1

## Goal

Restore Windows build support by enabling Crossterm's Windows feature, then publish the corrective Shelly v3.14.1 release.

## Scope

- Change only Crossterm dependency features required for Windows compilation.
- Verify the dependency configuration and release checks as far as the available environment permits.
- Commit, push, tag, and publish v3.14.1 without moving the failed v3.14.0 tag.

## Tasks

- [x] Enable Crossterm Windows support.
- [x] Validate the dependency configuration and release candidate.
- [ ] Publish Shelly v3.14.1 and verify the release workflow.

## Evidence

v3.14.0 Windows build and updater jobs failed while compiling Crossterm because `default-features = false` enabled only `events`, not `windows`.

`Cargo.toml` now preserves Crossterm `0.29` and `default-features = false`, with `features = ["events", "windows"]`. Root package metadata and `Cargo.lock` are aligned at `3.14.1`.

`cargo check --locked`, `cargo test --locked`, and `git diff --check` passed. The local GNU Windows cross-check reached the linker dependency stage but was blocked by a missing `x86_64-w64-mingw32-gcc`; no toolchain was installed. GitHub's native Windows runner remains the publication verification for this environment-specific target.
