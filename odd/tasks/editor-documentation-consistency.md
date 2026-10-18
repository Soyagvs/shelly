# Editor documentation consistency

## Scope

Align public documentation across Shelly, shelly-nvim, and shelly-vscode without changing runtime or grammar behavior.

## Verified claims

- `shelly-lsp` provides diagnostics, completion, hover, definition, references, document symbols, workspace symbols, and conservative rename.
- Both editor clients launch the same `shelly-lsp` executable through stdio.
- VS Code grammar coverage must remain described as Shelly v3.1; current tests do not prove v3.5 grammar coverage.

## Tasks

- [x] Remove stale core release wording and make integration navigation reciprocal.
- [x] Align Neovim integration documentation with verified LSP capabilities.
- [x] Align VS Code integration documentation with verified LSP capabilities while retaining the v3.1 grammar claim.
- [x] Verify links and release/version references.
- [x] Commit and push the documentation-only work on `main`.

## Evidence

- `README.md` no longer says that the published `v3.5.0` release is pending.
- Shared LSP capabilities are documented as diagnostics, completion, hover, definition, references, document/workspace symbols, and conservative rename.
- The VS Code grammar remains described as Shelly v3.1; no v3.5 grammar claim was added.
- README and editor-support quickstart link directly to both editor integrations.
- Published commits: Shelly `637e9a7`, shelly-nvim `23ca9bb`, and shelly-vscode `3961fee`.
- `git diff --check` passed in every repository; only generated `.codegraph/` remains untracked.
