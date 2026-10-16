# Shelly self-update release

Feature: publish the verified self-update feature and distribute the LSP editor client.

## Tasks

- [x] Inspect and add Windows runtime verification for updater hand-off.
- [x] Remove staged updater binaries after successful transactional replacement while preserving them for rollback recovery.
- [x] Make staged-binary cleanup best-effort so cleanup failure cannot report a successful replacement as failed.
- [ ] Commit and push the Shelly update and documentation work to `main`.
- [ ] Create and verify a Shelly release containing the updater.
- [ ] Commit, push, package, and publish the Shelly VS Code LSP client.

## Constraints

- Mason submission is explicitly excluded.
- Preserve unrelated `.codegraph/` directories as untracked local state.
- Use independently verified artifacts before publication.

## Verification evidence

- `cargo fmt --all -- --check` — passed locally.
- `cargo test --locked` — passed locally on non-Windows; the Windows-only hand-off test is cfg-disabled.
- GitHub Actions `windows-update-handoff` runs `cargo test --locked --test update windows_deferred_hand_off_replaces_binaries` on `windows-latest`.
- `cargo update -p shelly` — passed; updated the root package in `Cargo.lock` from `shelly v3.3.1` to `shelly v3.4.0` and reported 1 package locked, with 4 dependencies unchanged.
- `cargo fmt --check` — passed.
- `cargo test --locked` — passed; 3 unit tests, 201 CLI smoke tests, 6 LSP feature tests, 4 LSP smoke tests, 5 LSP symbol tests, 4 LSP syntax tests, 1 Mason packaging test, 2 Neovim LSP tests, 3 release packaging tests, and 6 update tests passed; doc-tests passed with 0 tests.
- `cargo check --locked` — passed; finished the dev profile check for `shelly v3.4.0`.
- Remote Windows CI failure: compilation failed because `src/update.rs` unconditionally compiled the Unix-only `std::os::unix::fs::PermissionsExt` import and `set_mode` call; the permission adjustment is now gated with `#[cfg(unix)]`.
- Remote Windows CI run 38904116685: `windows_deferred_hand_off_replaces_binaries` failed because staged files remained after paired replacement; successful replacement now removes staged sources, while rollback tests verify they remain after failure.
- Local validation after the fix: `cargo fmt --all` and `cargo test --locked` passed; Unix archive extraction still applies executable permissions, while Windows no longer compiles the Unix-only code.
- Cleanup-failure validation: staged cleanup now logs and retains an unremovable temporary path without failing the update; focused update tests cover this deterministically.
