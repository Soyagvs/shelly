# Release Shelly and editor integrations

Prepare and publish the authorized patch releases for the current tracked worktree changes. Exclude generated `.codegraph/` files.

## Specs

1. "ahora sube las releases nuevas de shelly , shelly-nvim y shelly-vscode"
2. "Publicar todo lo actual"
3. "Sí: 3.14.2 y 0.6.1"

## Tasks

- T1 · S1-S3 · Map version, artifact, changelog, and GitHub release workflows · inline.
- T2 · S1-S3 · Prepare, test, commit, tag, and publish Shelly v3.14.2 · release.
- T3 · S1-S3 · Prepare, test, commit, tag, and publish shelly-nvim and shelly-vscode v0.6.1 · release.
- T4 · S1-S3 · Verify release assets and tags across all repositories · verify.

## Log

L1 · "ahora sube las releases nuevas de shelly , shelly-nvim y shelly-vscode"
L2 · "Publicar todo lo actual"
L3 · "Sí: 3.14.2 y 0.6.1"
L4 · Release scope is every currently tracked worktree change in the three repositories; `.codegraph/` remains excluded.
L5 · Version metadata updated: Shelly 3.14.2; shelly-nvim and shelly-vscode 0.6.1. Core full test suite, formatting, and editor release checks pass before publication.
