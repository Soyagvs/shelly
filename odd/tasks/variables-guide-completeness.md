# Variables guide completeness

## Goal
Make `docs/variables.md` a complete, approachable reference for Shelly variables: valid and invalid forms, mutation, values, assignment behavior, and scope boundaries, with deeper collection/function details linked rather than duplicated.

## Scope
- `docs/variables.md`
- This task record

## Non-goals
- Change variable, constant, collection, command, or function behavior.
- Duplicate the full constants, lists, dictionaries, commands, or functions references.
- Commit, publish, or create a pull request without explicit user approval.

## Tasks

- [x] **VGC-1 — Audit variable semantics.** Map variable-specific behavior and corroborate every proposed claim against source/tests.
- [x] **VGC-2 — Complete core assignment guidance.** Documented naming, declaration, empty versus undefined values, assignment, reassignment, result binding, and rejected forms.
- [x] **VGC-3 — Explain values, mutation, and scope.** Added concise value, copy, collection-mutation, and function-scope guidance with links to the detailed references.
- [x] **VGC-4 — Verify the guide.** Ran Markdown, link, and focused executable checks for every new behavioral example.
  - Checks: `git diff --check -- docs/variables.md`; fence/local-link validation; isolated Cargo-run checks for identifiers, emptiness, assignment, reassignment, result binding, comma rejection, triple captures, copies, collection mutation, and `global` scope.

## Evidence
- Verification passed: all documented supported forms ran successfully and documented rejected forms were rejected. No behavioral claims remain unverified.
- The user requires each page to document both supported and unsupported forms, including mutation behavior.
- No commit requested; working changes remain uncommitted unless the user explicitly asks.
