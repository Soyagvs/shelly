# Reliable release badges

Feature: show release version and download counts in Shelly, Shelly VS Code, and Shelly Neovim READMEs without calling GitHub through Shields.io's shared token pool.

## Decisions

- Store badge payloads as repository-owned JSON under `.github/badges/`.
- Generate payloads with a GitHub Actions workflow on published releases, a daily schedule, and manual dispatch.
- Render badges through Shields.io's generic endpoint over raw GitHub content, avoiding Shields.io's GitHub API token pool.
- Initialize current values from the GitHub Releases API: Shelly `v3.4.0` with 6 downloads, Shelly VS Code `v0.0.8` with 0 downloads, and Shelly Neovim as not released with 0 downloads.
- Preserve the release archive/download links next to the badges.

## Tasks

- [x] Add owned badge data and update the Shelly README/workflow.
- [x] Add owned badge data and update the VS Code README/workflow.
- [x] Add owned badge data and update the Neovim README/workflow.
- [x] Verify badge payloads, workflows, and README links.
- [x] Commit the three repository changes.

## Evidence

- Added owned endpoint payloads and release/scheduled/manual updater workflows to all three repositories.
- Initialized payloads from the Releases API: Shelly `v3.4.0` / 6, VS Code `v0.0.8` / 0, and Neovim `unreleased` / 0.
- Replaced Shelly's token-pool-dependent GitHub badges and added matching badges to both editor README files.
- JSON payload validation passed; independent verification confirmed workflow triggers/permissions/fallbacks, raw-main endpoint URLs, and `git diff --check` in all three repositories.
- Committed the implementation as Shelly `bf34369`, VS Code `f4e2713`, and Neovim `9fbd7e5` (`feat(readme): add reliable release badges`).
