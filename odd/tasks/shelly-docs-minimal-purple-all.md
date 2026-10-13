# Shelly minimal purple docs refresh

Feature: make all public Shelly docs minimal, purple-accented, example-first, and rule-clear.

## Goals

- Apply the existing purple visual language beyond README and `docs/language.md`.
- Keep each doc short, scannable, and beginner-friendly.
- Lead with the common happy path.
- Include at least one useful Shelly code example per topic.
- Keep rules explicit and grouped.
- Preserve accurate behavior; do not invent language features.
- Avoid source behavior changes.
- Do not commit unless the user explicitly asks.

## Tasks

- [x] Map current docs and choose a small reusable page shape.
- [x] Rewrite focused docs with purple dividers, concise explanations, examples, and rules.
- [x] Update README/language links only if needed.
- [x] Verify markdown links and examples structurally.
- [x] Run relevant project checks.

## Evidence

- Rewrote all 15 `docs/*.md` files with a consistent minimal page shape: title, purple divider, short purpose, Shelly example, clear rules, and bottom purple divider.
- Kept README links unchanged; `docs/language.md` remains the focused index.
- Markdown link check passed: 16 markdown files checked.
- Structural check passed: every `docs/*.md` file contains the purple divider and at least one `shelly` code block.
- GREEN: `cargo fmt --check` passed.
- GREEN: `cargo test` passed: 80 tests.
- GREEN: `printf 'yes\\nblue\\n' | cargo run --quiet -- example-01.shelly` passed; final output included `Status: count is high enough`.
