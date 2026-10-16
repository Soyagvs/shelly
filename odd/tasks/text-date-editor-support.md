# Document and surface text/date helpers

Feature: make `substring`, `padLeft`, `padRight`, and `formatDate` discoverable in Shelly documentation and supported across LSP, VS Code, and Neovim editor integrations.

## Decisions

- Add a dedicated date-formatting guide and link it from helpers, language reference, README, and relevant editor-facing documentation.
- Add helper completion/hover detail to the LSP where its capability model supports standard helpers.
- Add the helpers to VS Code TextMate builtin highlighting and structural tests.
- Add the helpers to Neovim syntax highlighting and structural verification.
- Keep `formatDate` semantics consistent everywhere: strict UTC ISO input and `YYYY`, `MM`, `DD`, `HH`, `mm`, `ss` patterns.

## Tasks

- [x] Audit all documentation and editor/LSP helper surfaces.
- [x] Document the helpers and date formatting in Shelly.
- [x] Add LSP discoverability support.
- [x] Add VS Code and Neovim highlighting support.
- [x] Verify all integration updates.
- [ ] Publish Shelly `v3.4.2`, VS Code `v0.1.0`, and Neovim `v0.1.1` so users receive them.

## Evidence

- Audit confirmed runtime/docs exist, but LSP lacks helper metadata/completion/hover; VS Code and Neovim builtin lists omit all four helpers.
- Added canonical UTC date-formatting guide and linked it from helper, language, and README navigation.
- Added LSP helper metadata plus completion/hover coverage; focused LSP tests passed (8 tests).
- Added all four helpers to VS Code and Neovim builtin highlighting and linked their READMEs to canonical documentation; VS Code tests and a headless Neovim syntax check passed.
- Independent cross-repository verification passed: Shelly full/LSP tests; VS Code validate/build/tests; Neovim headless syntax load; documentation and grammar checks; and diff checks in all repositories.
