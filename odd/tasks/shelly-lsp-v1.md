# Shelly complete LSP v1

Feature: static project-aware LSP with diagnostics, completion, hover, navigation, references, rename, and symbols. Formatter excluded.

## Decisions

- LSP v1 must be complete for its declared static scope; unsafe/dynamic rename and navigation must refuse rather than guess.
- Build source spans/AST, symbol scopes, and module graph before feature handlers.
- Open editor buffers override on-disk workspace files.
- Keep interpreter and CLI behavior compatible; LSP never executes Shelly scripts.
- Formatting is explicitly out of scope.

## Tasks

- [x] Build LSP document/position protocol foundation.
- [x] Add loss-tolerant spanned static syntax parser.
- [x] Build scopes, symbols, references, and module workspace graph.
- [x] Implement completion, hover, definition, references, rename, and symbols.
- [x] Document full LSP v1 and verify all suites.

## Evidence

- LSP suites: 19 passed; CLI smoke suite: 201 passed; full suite: 216 passed.
- `cargo fmt --check`, `cargo check`, and `git diff --check`: passed with no compiler warnings.
- No commit created; awaiting explicit user authorization.
