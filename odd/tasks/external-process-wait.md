# External Process Wait

## Objective
Add a Shelly construct that keeps the interpreter alive while an external process emits output, dispatching each output line to a sequential handler, and keep every supported editor integration aligned with that syntax.

## Delivered runtime scope
- Added `watch <program> <args...> -> $line ... end`.
- The watcher streams stdout lines in arrival order to a serial handler and ends with its child.
- Added bare `wait`, which blocks until normal process interruption.
- Stderr is not an event source.

## Delivered editor and LSP scope
- Shelly docs describe both constructs and their lifecycle semantics.
- LSP recognizes valid watcher/wait forms, rejects invalid wait arguments and malformed/empty watcher destinations, without adding semantic-token capabilities.
- Neovim syntax highlights `watch` and `wait`; its README documents both.
- VS Code grammar recognizes both, `watch` opens an indentation block, regression tests and README are updated.

## Constraints and non-goals
- No general concurrency, task scheduler, file watcher, socket listener, child restart, or multiple simultaneous activities.
- No semantic-token provider was added; it is separate scope.
- Commits and push were explicitly authorized by the user; each repository uses a focused feature branch.
- Generated `.vsix` artifacts in `shelly-vscode` were left untouched.

## Delivery
- Strategy: ask-on-risk.
- Revised forecast: approximately 500 authored changed lines; future commit planning must select review slices because this exceeds the preferred review budget.
- Shelly work-unit commit: `f367ebb79d0a136887c2b3615a02d552a9f78828` — `feat: add process watch command` on `feat/process-watch`.
- Neovim companion commit: `7b0ffd8cabda495d24e8789cc1cf0bf73dd28ce2` — `feat: highlight process watch commands` on `feat/process-watch-support`.
- VS Code companion commit: `937acedce631d18d120e8154df0a4f0e8007fd85` — `feat: support process watch commands` on `feat/process-watch-support`.

## Tasks
- [x] EPW-1 — Implement and test sequential external-process waiting.
- [x] EPW-2 — Document the external-process wait construct in `docs/commands.md`.
- [x] EPW-3 — Complete Shelly documentation and LSP regression coverage.
- [x] EPW-4 — Update the sibling Neovim integration.
- [x] EPW-5 — Update the sibling VS Code integration.
- [x] EPW-6 — Reject an empty `watch` destination variable.
  - Outcome: bare `$` now produces the existing invalid-watch-destination diagnostic while `$name` remains valid.
  - Evidence: focused RED/GREEN regression; independent verification and full LSP suite passed.

## Verification evidence
- Shelly CLI: `cargo test --test cli_smoke` — 217 passed, 0 failed.
- Shelly LSP: `cargo test --test lsp_syntax --test lsp_features --test lsp_smoke` — 20 passed, 0 failed.
- Parent LSP spot check: `cargo test --test lsp_features watch_and_wait_forms_are_diagnosed_without_extra_capabilities -- --exact` — 1 passed.
- Neovim: independent headless syntax-load verification passed.
- VS Code: independent `npm test` — 14 passed; `npm run validate` passed. Worker also reported successful build, package, and package verification.
- Native review preflight and assessment were unavailable for all three repositories because the verified package-local Gentle AI binary is missing; independent verification was run instead.

## Next step
All focused feature branches were pushed to their respective `origin` remotes. Pull-request creation and merge remain user decisions.
