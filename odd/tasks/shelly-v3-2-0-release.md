# Shelly v3.2.0 release

Feature: prepare and publish Shelly v3.2.0 after adding migration helper commands and project-structure docs.

## Scope

- Bump package/docs version from v3.1.1 to v3.2.0.
- Run full release verification.
- Commit version bump, merge release branch to main, push main, tag v3.2.0, and push the tag to trigger release binaries.

## Tasks

- [x] Inspect Shelly release workflow.
- [x] Prepare v3.2.0 release changes.
- [x] Run full release verification.
- [ ] Tag and push release.

## Evidence

- Release workflow is tag-triggered by `.github/workflows/release.yml` for `v*` tags and uploads Linux, macOS, and Windows binaries.
- Full release verification PASS: `cargo fmt --check`; `cargo test` (154 integration tests passed); `cargo build --release --locked`; `cargo run --quiet -- examples/conditionals.shelly`; no active `v3.1.1` references outside archival `odd/tasks/**`.
- Native review inspect blocked before release because the package-local Gentle AI binary is missing (`package-local-binary-missing`).

- Version-only release preparation completed: `Cargo.toml` package version bumped from `3.1.1` to `3.2.0`.
- `Cargo.lock` Shelly package entry updated to `3.2.0`.
- README latest release text updated from `v3.1.1` to `v3.2.0`.

