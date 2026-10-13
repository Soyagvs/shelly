# Shelly install script and v3.0.0 release

Feature: add a simple Linux/macOS installer and publish the next public release as SemVer `v3.0.0`.

## Decisions

- Use SemVer tag `v3.0.0` instead of `v3.0`, because SemVer requires `MAJOR.MINOR.PATCH`.
- Update Cargo package version to `3.0.0` before tagging.
- Add `install.sh` for Linux x86_64, macOS x86_64, and macOS arm64.
- Installer downloads release artifacts from GitHub Releases, unpacks them, installs `shelly`, and prints next steps.
- Prefer a user-writable install path by default; allow `INSTALL_DIR` override.
- Publish by committing installer/version/docs, pushing `main`, then tagging and pushing `v3.0.0`.

## Tasks

- [x] Add Linux/macOS `install.sh`.
- [x] Update README install instructions for non-technical users.
- [x] Bump Cargo version to `3.0.0`.
- [x] Verify script syntax and project checks.
- [x] Commit and push changes.
- [x] Create and push `v3.0.0` tag.

## Evidence

- Added `install.sh` for Linux x86_64, macOS x86_64, and macOS arm64.
- Updated `README.md` to make Linux/macOS installation a single `curl ... | sh` command and to use a concrete `hello.shelly` example.
- Bumped `Cargo.toml` package version from `0.1.0` to `3.0.0` for the `v3.0.0` release.
- GREEN: `cargo check --quiet` passed after the version bump.
- GREEN: `sh -n install.sh && cargo fmt --check && cargo test && printf 'yes\\nblue\\n' | cargo run --quiet -- example-01.shelly` passed; full suite reported 84 tests passed.
- Committed and pushed `f278793` with message `ci: add unix installer for v3 release`.
- Created and pushed SemVer tag `v3.0.0`, which triggers the GitHub Release binary workflow.
