# Multi-repository release 2026-10

## Approved scope

Prepare, commit, push, tag, and publish the approved releases while excluding generated `.codegraph/` state:

- Shelly `v3.5.0`
- shelly-nvim `v0.1.2`
- shelly-vscode `v0.1.1`, including the packaged VSIX attached to its GitHub release

## Tasks

- [x] Keep all work on `main`; temporary release branches were removed at the user's request.
- [x] Prepare and verify Shelly `v3.5.0` metadata and documentation.
- [ ] Prepare and verify shelly-nvim `v0.1.2`.
- [ ] Prepare, package, and verify shelly-vscode `v0.1.1`.
- [ ] Push, tag, and publish each approved release.

## Evidence

- Shelly metadata prepared: `Cargo.toml` and `Cargo.lock` declare `3.5.0`; `README.md` identifies `v3.5.0` as the pending publication.
- Shelly implementation and documentation committed on `main`: `620cfa5` (`feat: add command, case, and regex syntax`).
- Verification passed: `cargo fmt --check`, `cargo test` (253 passed), sample execution, `cargo build --release --locked`, Linux release archive verification, and SHA-256 verification.
- Native review could not run because the package-local Gentle AI binary is unavailable; an independent release verifier completed the checks above.
