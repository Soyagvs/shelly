# Shelly public release readiness

Feature: make Shelly public-facing with license, latest-release README notes, and public repository visibility.

## Decisions

- Use the MIT License: permissive, familiar, and simple for a small language/tool.
- Add a `Latest release` section to README and keep it updated for each release.
- Make the GitHub repository public after verification.
- Check the `v3.0.0` release assets before announcing public availability.

## Tasks

- [x] Add MIT `LICENSE` and update license badge.
- [x] Add latest release notes to `README.md`.
- [ ] Verify release assets and fix any missing platform artifact if needed.
- [x] Run local checks.
- [ ] Commit and push public-readiness changes.
- [ ] Create and push public `v3.0.1` patch tag.
- [ ] Make repository public.

## Evidence

- Added MIT `LICENSE` and updated the README license badge from private to MIT.
- Added a `Latest release` section to README and set the public-ready release line to `v3.0.1` so the public tag includes the license and latest install instructions.
- Observed `v3.0.0` release workflow still in progress while some assets were already uploaded; `v3.0.1` will be the public patch release after license/readme changes.
- GREEN: `cargo check --quiet && cargo fmt --check && cargo test && printf 'yes\\nblue\\n' | cargo run --quiet -- example-01.shelly` passed; full suite reported 84 tests passed.
