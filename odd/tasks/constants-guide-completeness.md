# Constants guide completeness

## Goal
Make `docs/constants.md` a complete, approachable guide to Shelly constants: valid and rejected forms, immutable behavior, values, scope, and module boundaries, without duplicating detailed topic references.

## Scope
- `docs/constants.md`
- This task record

## Non-goals
- Change constant, variable, collection, command, function, or module behavior.
- Duplicate the full variables, lists, dictionaries, commands, functions, or modules references.
- Commit, publish, or create a pull request without explicit user approval.

## Tasks

- [x] **CGC-1 — Audit constant semantics.** Map constant-specific behavior and corroborate all proposed claims against source/tests.
- [x] **CGC-2 — Complete core constant guidance.** Documented naming, declaration, values, expressions, undefined reads, and accepted/rejected forms.
- [x] **CGC-3 — Explain immutable boundaries.** Covered collection mutation, capture destinations, scope, and import/export boundaries with concise reference links.
- [x] **CGC-4 — Verify the guide.** Ran Markdown, link, and focused executable checks for all new behavioral examples.
  - Checks: `git diff --check -- docs/constants.md`; fence/local-link validation; isolated Cargo-run checks for identifiers, declarations, values, expressions, immutable collection operations, capture rejection, scope, and import/export visibility.

## Evidence
- Verification passed: all documented supported forms ran successfully and every documented rejected form was rejected. No behavioral claims remain unverified.
- The user requires complete documentation of supported and rejected forms, including mutation behavior.
- No commit requested; working changes remain uncommitted unless the user explicitly asks.
