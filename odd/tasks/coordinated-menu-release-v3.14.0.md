# Coordinated Menu Release v3.14.0

## Goal

Publish the completed Shelly declarative-menu work and matching editor syntax support from each repository's `main` branch.

## Authorized scope

- Shelly: complete the style-context guide, commit all pending product/docs/test/ODD changes, publish `v3.14.0`.
- shelly-nvim: commit pending syntax, tests, and README changes; publish `v0.6.0`.
- shelly-vscode: commit pending grammar, tests, README, and ODD changes; publish `v0.6.0`.
- Exclude local `.codegraph/` metadata from all commits.

## Tasks

- [x] Complete and verify the Shelly style-context guide.
- [ ] Validate, commit, push, and release Shelly v3.14.0.
- [ ] Validate, commit, push, and release shelly-nvim v0.6.0.
- [ ] Validate, commit, push, and release shelly-vscode v0.6.0.

## Evidence

Shelly validation passed before version alignment: `cargo fmt --check`, `cargo test` (395 tests), conditionals example, `release_packaging` (5), `update` (12), `installers` (2), and `git diff --check`.

Work-unit commits prepared on `main`:

- `9562663 docs(style): document styled output contexts`
- `f0ff732 feat(menu): simplify declarative menu rendering`

`Cargo.toml` and `Cargo.lock` are aligned at `3.14.0`; `cargo check --quiet` passed.
