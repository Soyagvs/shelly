# Shelly release binaries

Feature: prepare Shelly for non-technical users with downloadable release binaries.

## Goals

- Build release binaries automatically on GitHub Actions.
- Publish platform artifacts on GitHub Releases.
- Keep installation instructions simple: download, unpack, run `shelly`.
- Avoid requiring Rust or Cargo for end users.
- Do not publish a release until the user explicitly asks.

## Decisions

- Start with GitHub Releases as the distribution channel.
- Build Linux x86_64, macOS x86_64, macOS arm64, and Windows x86_64.
- Package Unix artifacts as `.tar.gz` and Windows as `.zip`.
- Trigger release packaging from version tags like `v0.1.0`.
- Keep advanced installers/updaters for a later step.

## Tasks

- [x] Add CI workflow for formatting, tests, and showcase execution.
- [x] Add release workflow that builds and uploads platform binaries for tags.
- [x] Add minimal install/release docs for non-technical users.
- [x] Verify workflow YAML and local checks.
- [x] Commit and push when verified.

## Evidence

- Added `.github/workflows/ci.yml` for pull requests and pushes to `main` with formatting, tests, and showcase execution.
- Added `.github/workflows/release.yml` for `v*` tags with Linux x86_64, macOS x86_64, macOS arm64, and Windows x86_64 release binaries.
- Release artifacts are packaged as `.tar.gz` for Unix and `.zip` for Windows, then uploaded to the tag's GitHub Release with `softprops/action-gh-release`.
- Updated `README.md` with a non-technical Install section focused on GitHub Release downloads and unpack/run steps.
- Local validation run on this worktree:
  - `cargo fmt --check` — passed.
  - `cargo test` — passed.
  - `printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly` — passed.
  - Lightweight `python3` workflow structure check — passed for `.github/workflows/ci.yml` and `.github/workflows/release.yml`.
- Re-verified before delivery: `cargo fmt --check`, `cargo test`, and the showcase command passed.
- Committed and pushed as `ci: add release binary workflows`.
