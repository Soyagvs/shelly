# Release Shelly and editor integrations

Prepare and publish the authorized patch releases for the current tracked worktree changes. Exclude generated `.codegraph/` files.

## Specs

1. "ahora sube las releases nuevas de shelly , shelly-nvim y shelly-vscode"
2. "Publicar todo lo actual"
3. "Sí: 3.14.2 y 0.6.1"
4. "y no actualizaste la doc, con clear, con progress y lo que hicimos"
5. "si"

## Tasks

- T1 · S1-S3 · Done: map version, artifact, changelog, and GitHub release workflows · inline.
- T2 · S1-S3 · Done: prepare, test, commit, tag, and publish Shelly v3.14.2 · release.
- T3 · S1-S3 · Done: prepare, test, commit, tag, and publish shelly-nvim and shelly-vscode v0.6.1 · release.
- T4 · S1-S3 · In progress: verify release assets and tags across all repositories · verify.
- T5 · S4-S5 · In progress: document `clear`, `progress`, contextual LSP completion, module routes, and purple icons · inline.
- T6 · S4-S5 · Pending: prepare, test, tag, publish, and verify Shelly v3.14.3 and editor v0.6.2 documentation releases · release.

## Log

L1 · "ahora sube las releases nuevas de shelly , shelly-nvim y shelly-vscode"
L2 · "Publicar todo lo actual"
L3 · "Sí: 3.14.2 y 0.6.1"
L4 · Release scope is every currently tracked worktree change in the three repositories; `.codegraph/` remains excluded.
L5 · Version metadata updated: Shelly 3.14.2; shelly-nvim and shelly-vscode 0.6.1. Core full test suite, formatting, and editor release checks pass before publication.
L6 · "y no actualizaste la doc, con clear, con progress y lo que hicimos"
L7 · "si"
L8 · Public release tags are immutable for this release cycle. The authorized correction is a documentation patch release: Shelly 3.14.3 and editor integrations 0.6.2.
