# Language guide examples

## Goal
Make `docs/language.md` Shelly's practical language overview: teach the core workflow with concise, executable examples while leaving complete contracts to the topic reference pages.

## Scope
- `docs/language.md`
- This task record

## Non-goals
- Change runtime behavior, tests, commands, or examples outside the guide.
- Duplicate full topic-page references.
- Expand the README beyond its landing-page role.
- Commit, publish, or create a pull request without explicit user approval.

## Tasks

- [x] **LGE-1 — Audit learning gaps.** Map teaching gaps and validate candidate syntax against current documentation, examples, and tests.
- [x] **LGE-2 — Add guided language examples.** Added a compact automation walkthrough, control-flow progression, value/data flow example, and command-versus-shell contrast.
  - Outcome: the guide now introduces common language patterns before routing to detailed references.
- [x] **LGE-3 — Improve reference navigation.** Added task-oriented routes to the existing CLI/input, files, reusable-code, failure, and editor-support pages.
- [x] **LGE-4 — Verify the guide.** Ran Markdown, link, and focused executable-example checks.
  - Checks: `git diff --check -- docs/language.md`; local-link and fence validation; isolated execution of each new Shelly snippet through `cargo run --quiet`.

## Evidence
- Verification passed: the new automation, control-flow, values/JSON, and command/shell snippets ran successfully. The shell pipeline was verified on the current Unix environment only.
- The README intentionally remains a landing page; this guide owns syntax-oriented teaching examples.
- No commit requested; working changes remain uncommitted unless the user explicitly asks.
