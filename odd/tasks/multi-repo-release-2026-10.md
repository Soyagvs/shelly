# Multi-repository release 2026-10

## Approved scope

Prepare, commit, push, tag, and publish the approved releases while excluding generated `.codegraph/` state:

- Shelly `v3.5.0`
- shelly-nvim `v0.1.2`
- shelly-vscode `v0.1.1`, including the packaged VSIX attached to its GitHub release

## Tasks

- [ ] Create isolated release branches.
- [x] Prepare Shelly `v3.5.0` release metadata and documentation; verification deferred by request.
- [ ] Prepare and verify shelly-nvim `v0.1.2`.
- [ ] Prepare, package, and verify shelly-vscode `v0.1.1`.
- [ ] Commit, push, tag, and publish each approved release.

## Evidence

- Shelly metadata prepared: `Cargo.toml` and `Cargo.lock` now declare `3.5.0`; `README.md` identifies `v3.5.0` as the pending publication.
