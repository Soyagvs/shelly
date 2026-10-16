# Fix release badge workflow publishing

Feature: make release-triggered badge workflows check out `main` before committing so their generated payloads are pushed successfully rather than failing from detached HEAD.

## Decision

- Pin workflow checkout to `main` in all three repositories.
- This preserves the release tag as the trigger while directing generated badge-data commits to the default branch.
- Rerun or manually dispatch the Neovim workflow after publishing the correction so `v0.1.0` is reflected in its badge.

## Tasks

- [x] Fix checkout refs in Shelly, Shelly VS Code, and Shelly Neovim workflows.
- [x] Verify workflow structure and publish the three corrections.
- [x] Refresh the Neovim v0.1.0 badge payload.

## Evidence

- Neovim release-badge run `38971947003` committed the generated data but failed on `git push` because checkout was detached at the release tag.
- Shelly already specified `ref: main`; added that checkout ref to the VS Code and Neovim workflows.
- Independent verification confirmed checkout happens on `main` before commit/push; published the correction to all three repositories.
- Manual Neovim workflow run `38972562197` passed and committed `b6af8b4`, changing its release badge to `v0.1.0`.
