# Link release publication to badge updates

Feature: make Shelly release badges update as part of every successful binary release rather than relying on an event that may not be emitted by a release created with the workflow token.

## Decisions

- Run badge refresh as a final release-workflow job that depends on every platform build succeeding.
- Keep the separate badge workflow for scheduled and manual download-count refreshes.
- Refresh the current release badge to `v3.4.3` as part of the repair.

## Tasks

- [x] Add a release-workflow badge-refresh job and update its tests/documentation.
- [x] Verify workflow structure and badge payload correctness.
- [ ] Commit, publish, and confirm the linked behavior.

## Evidence

- `.github/badges/release.json` remains at `v3.4.0` while `Cargo.toml` is `3.4.3`.
- The current badge workflow listens for `release: published`, but the release workflow creates releases with `GITHUB_TOKEN`; this does not create a subsequent workflow run for ordinary events.
- Badge updates now run as the release workflow's final job after the complete build matrix, while scheduled/manual refreshes share the same script.
