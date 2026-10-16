# Install Shelly LSP with the CLI

Feature: make fresh Shelly installations place both `shelly` and `shelly-lsp` in the configured install directory on Linux/macOS and Windows.

## Decisions

- Treat the release archive as the single source for both executables.
- Fail before copying when either expected executable is absent.
- Preserve existing file-icon and PATH behavior.
- Add static regression coverage that verifies both installer scripts extract and install `shelly-lsp`.

## Tasks

- [x] Update Unix and Windows installers to install `shelly-lsp` atomically with Shelly.
- [x] Add regression coverage for both installer scripts.
- [x] Run focused and full verification.
- [x] Commit the installer fix.

## Evidence

- Updated `install.sh` and `install.ps1` to validate and install the LSP executable alongside the CLI.
- Added `tests/installers.rs`; focused `cargo test --test installers` passed (1 test).
- No meaningful RED observation was recorded because the pre-existing scripts did not have installer coverage.
- Independent verification passed: `cargo test --test installers`, full `cargo test`, and `git diff --check`; both installers validate and install the LSP before reporting success.
- Installer fix committed as `5e22fc3a2b26b8a181736023d07692cb5aa6757e` (`fix(installer): install shelly-lsp with Shelly`).
