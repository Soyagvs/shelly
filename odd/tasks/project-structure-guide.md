# Project structure guide

## Goal
Make `docs/project-structure.md` clear for both new and experienced Shelly users: explain when a project is useful, how to run it, and how to grow its structure without unnecessary complexity.

## Scope
- `docs/project-structure.md`
- This task record

## Non-goals
- Change the scaffold, runtime behavior, manifest format, or module semantics.
- Duplicate the full commands, tests, or modules references.
- Add more purple dividers; retain the established visual style.
- Commit, publish, or create a pull request without explicit user approval.

## Tasks

- [x] **PSG-1 — Audit clarity and accuracy.** Identify reader-facing gaps and validate the affected behavior against implementation evidence.
- [x] **PSG-2 — Clarify starting and running a project.** Added plain-language guidance for single scripts versus projects, optional scaffold folders, and command choices.
- [x] **PSG-3 — Improve examples and growth guidance.** Replaced the non-self-contained test with a runnable project-context example; clarified path resolution, folder roles, import/include intent, and the growth progression.
- [x] **PSG-4 — Verify the guide.** Ran Markdown, link, source-consistency, and focused executable-example checks.
  - Checks: `git diff --check -- docs/project-structure.md`; link/fence validation; isolated `shelly test`, direct execution, `shelly run`, explicit and manifest checks, and relative include execution.

## Evidence
- Verification passed: no whitespace, link, or fence issues; all checked snippets and command paths behaved as documented.
- Existing purple dividers are retained; the documentation style must remain clear for technical and non-technical readers.
- No commit requested; working changes remain uncommitted unless the user explicitly asks.
