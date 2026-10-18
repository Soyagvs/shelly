# Recursive file globs

Feature: support recursive `**` matching in Shelly file globs.

## Decisions

- `*` matches only within one directory segment.
- `**` matches zero or more directory segments, so `**/*.shelly` includes matching files in the current directory and all descendants.
- Results are deterministic and sorted.
- No match returns an empty list.
- Recursive matching does not follow symlinks.
- Globs return files only; directory selection is deferred.
- Runtime, LSP, VS Code, Neovim, and all relevant documentation must remain aligned.

## Tasks

- [x] Map current glob runtime, checker, LSP/editor integration points, and test conventions.
  - Evidence: runtime explicitly rejects `**`; checker/LSP already accept the pattern syntax without expansion. VS Code and Neovim only transport the Shelly LSP and require compatibility validation, not semantic implementations.
- [x] Implement recursive glob semantics with focused RED/GREEN coverage.
  - Evidence: RED observed the former explicit recursive-glob rejection; focused CLI and LSP tests passed after implementation, covering zero-depth/nested matches, sorting, symlink exclusion, empty results, checker acceptance, and diagnostics.
- [x] Update LSP/editor integrations and documentation.
  - LSP behavior is covered through checker/diagnostic tests; VS Code and Neovim have no local glob evaluator. Updated `docs/files.md`, `docs/editor-support.md`, `../shelly-vscode/README.md`, and `../shelly-nvim/README.md` to identify `shelly-lsp` as the semantic provider.
- [x] Run cross-repository verification.
  - Shelly: focused CLI/LSP tests, full `cargo test`, `cargo fmt --check`, and `git diff --check` passed. VS Code: validation, build, and 12 tests passed. Neovim: syntax and LSP configuration loaded headlessly.

## Evidence

- Product design agreed before implementation; `**/*.shelly` includes matches in the current directory and descendants.
- No commit was created: the user has not authorized commits.
