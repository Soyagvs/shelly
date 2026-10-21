# Lists guide completeness

## Goal
Make `docs/lists.md` a complete, approachable guide to Shelly lists: creation, access, selectors, filters, truthiness, copying, iteration, mutation, and rejected forms.

## Scope
- `docs/lists.md`
- This task record

## Non-goals
- Change collection parser, selector/filter, helper, loop, or mutation behavior.
- Duplicate full variables, constants, dictionaries, conditionals, loops, or JSON references.
- Commit, publish, or create a pull request without explicit user approval.

## Tasks

- [x] **LGC-1 — Audit list semantics.** Map list-specific behavior and corroborate claims against source/tests.
- [x] **LGC-2 — Complete list access and value guidance.** Documented creation/copies/truthiness, indexing, selectors, ranges, filters, iteration, and rejected forms.
- [x] **LGC-3 — Complete mutation guidance.** Reconciled mutation forms, nested paths, scalar-value limits, constants, and errors.
- [x] **LGC-4 — Verify the guide.** Full focused list verification passed: `cargo test --lib`, `cargo test --test cli_smoke`, Markdown links/fences, and `git diff --check`.

## Evidence
- The user requires complete documentation of supported and rejected forms, limits, and mutation behavior.
- No commit requested; working changes remain uncommitted unless the user explicitly asks.
