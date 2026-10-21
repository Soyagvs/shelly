# README landing page

## Goal
Turn `README.md` into Shelly's concise landing page: banner, product presentation, installation, minimal first-project path, and a documentation-index entry point.

## Scope
- `README.md`
- This task record

## Non-goals
- Duplicate language, test, editor, platform, release, or development reference documentation in the README.
- Change Shelly runtime behavior, tests, release assets, or editor setup guides.
- Commit, publish, or create a pull request without explicit user approval.

## Tasks

- [x] **RLP-1 — Map landing scope.** Confirm the documentation index is the canonical route for detailed topics.
  - Evidence: `docs/index.md` already covers getting started, language, library, automation, and editor routes.
- [x] **RLP-2 — Reduce README to landing content.** Retained banner, presentation, installation, a minimal first project, and direct links to the documentation index; removed detailed reference material and the earlier added examples.
  - Outcome: `README.md` is now a concise landing page.
- [x] **RLP-3 — Verify the landing page.** Ran proportional Markdown and link-structure checks.
  - Checks: `git diff --check -- README.md`; Markdown fence balance; internal-link and navigation-anchor validation; existence checks for `docs/index.md`, `docs/project-structure.md`, and `LICENSE`.

## Evidence
- Verification passed: no whitespace errors, broken links, broken anchors, or retained detailed reference sections were found.
- The earlier example-expansion approach was superseded by the user's landing-page direction.
- No commit requested; working changes remain uncommitted unless the user explicitly asks.
