# Shelly Windows installer

Feature: add a simple PowerShell installer for Windows users.

## Decisions

- Add `install.ps1` as the Windows equivalent of `install.sh`.
- Install to `%LOCALAPPDATA%\Shelly\bin` by default.
- Allow `SHELLY_VERSION` and `SHELLY_INSTALL_DIR` environment variable overrides.
- Download `shelly-windows-x86_64.zip` from GitHub Releases.
- Add the install directory to the current user's PATH when missing.
- Update README to show one-line PowerShell install.
- Publish as patch version `v3.0.2` because installation docs/scripts changed after `v3.0.1`.

## Tasks

- [x] Add `install.ps1`.
- [x] Update README Windows install instructions and latest release line.
- [x] Bump package version to `3.0.2`.
- [x] Verify scripts and project checks.
- [x] Commit and push changes.
- [x] Create and push `v3.0.2` tag.

## Evidence

- Added `install.ps1` for 64-bit Windows. It downloads `shelly-windows-x86_64.zip`, installs to `%LOCALAPPDATA%\\Shelly\\bin` by default, and adds that directory to the user's PATH when missing.
- Updated `README.md` with the Windows one-line installer: `irm https://raw.githubusercontent.com/Soyagvs/shelly/main/install.ps1 | iex`.
- Bumped `Cargo.toml` package version from `3.0.1` to `3.0.2`.
- GREEN: `cargo check --quiet` passed after the version bump.
- Local `pwsh` was unavailable, so `install.ps1` received a structural check for expected PowerShell installer operations.
- GREEN: `sh -n install.sh && cargo fmt --check && cargo test && printf 'yes\\nblue\\n' | cargo run --quiet -- example-01.shelly` passed; full suite reported 84 tests passed.
- Committed and pushed `9fcd477` with message `ci: add windows installer`.
- Created and pushed SemVer tag `v3.0.2`, which triggers the release binary workflow.
