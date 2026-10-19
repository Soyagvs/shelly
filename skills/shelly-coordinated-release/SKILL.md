---
name: shelly-coordinated-release
description: "Trigger: Shelly release, version, tag, or publish requests. Coordinate gated releases across Shelly, Neovim, and VS Code."
license: "MIT"
metadata:
  author: "Gentle AI"
  version: "1.0.0"
---

## Activation Contract

Load for any request to version, release, tag, publish, or attach release artifacts for Shelly or its Neovim and VS Code integrations.

## Hard Rules

- Require explicit user authorization before any commit, push, tag, GitHub release, asset publication, or publish command.
- Preflight all three repositories: clean working trees, `main` checked out, and a declared SemVer rationale for every version.
- Prepare and validate every release commit before creating any tag; never publish a partial release set.
- Shelly CI/release checks, Neovim headless syntax and changelog checks, and VS Code validation, build, tests, package, VSIX verification, and exact artifact attachment must pass.
- Tags must be exact `vX.Y.Z` tags pointing at the intended release commits. Verify GitHub releases and assets after publication.
- Treat badge automation as follow-up only; do not skip or weaken release gates after a failure or skip.

## Decision Gates

| Condition | Action |
| --- | --- |
| Any preflight or release gate fails/skips | Stop all publishing; report the failed/skipped gate. |
| A version lacks a SemVer rationale | Stop and obtain the rationale before preparation. |
| Required authorization is absent | Prepare no irreversible operation; ask for explicit authorization. |
| All gates pass and authorization exists | Commit/push, tag exact release commits, publish in dependency order, then verify releases/assets. |

## Execution Steps

1. Preflight Shelly, shelly-nvim, and shelly-vscode on clean `main`; record versions, SemVer rationale, and expected release commits.
2. Prepare all release commits and run every repository-specific gate before any tag or publication.
3. Confirm authorization, then commit/push and create exact `vX.Y.Z` tags at the validated release commits.
4. Publish all releases and attach the exact verified VSIX artifact; verify remote release metadata and assets.
5. Schedule badge automation as follow-up and report every command, gate, artifact path, and verification result.

## Output Contract

Return repository preflight, SemVer rationale, release commits, gate results, authorization status, exact tags, publication results, verified release URLs/assets, VSIX path, and badge follow-up status. Explicitly name any failed or skipped gate and confirm that no partial publication occurred.

## References

- `odd/tasks/coordinated-process-wait-release.md`
- `skills/shelly-language-integration/SKILL.md`
