# Strings guide completeness

## Goal
Make `docs/strings.md` a complete, approachable guide to Shelly strings: literals, escapes, interpolation, multiline behavior, scalar limits, and rejected forms.

## Scope
- `docs/strings.md`
- This task record

## Non-goals
- Change parser, string, interpolation, collection, command, or file behavior.
- Duplicate detailed variables, constants, lists, dictionaries, comments, commands, or files references.
- Commit, publish, or create a pull request without explicit user approval.

## Tasks

- [x] **SGC-1 — Audit string semantics.** Map string-specific behavior and corroborate proposed claims against source/tests.
- [x] **SGC-2 — Complete literal and escape guidance.** Documented quoted and multiline forms, valid/rejected escapes, delimiters, and limits.
- [x] **SGC-3 — Complete interpolation and value guidance.** Explained supported selectors, scalar boundaries, failures, common contexts, and cross-links.
- [x] **SGC-4 — Verify the guide.** Ran Markdown, link, and focused executable checks for all new behavioral examples.
  - Checks: `git diff --check -- docs/strings.md`; fence/local-link validation; isolated Cargo-run checks for literals, escapes, multiline delimiters, interpolation, scalar rejection, output/file contexts, and multiline function arguments.

## Evidence
- Verification passed: supported forms ran successfully and documented rejected forms were rejected. A multiline function-call example was corrected after verification and rechecked successfully.
- The user requires complete documentation of supported and rejected forms, including limits and practical behavior.
- No commit requested; working changes remain uncommitted unless the user explicitly asks.
