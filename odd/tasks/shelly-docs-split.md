# Shelly docs split

Feature: make README clearer and split language topics into separate documentation files.

## Goals

- Keep README concise and easy to scan.
- Add clear section links from README to per-topic docs.
- Split current language guide content into focused docs by feature.
- Keep `docs/language.md` as an index/overview that links to focused docs.
- Do not change Shelly language behavior.

## Docs created

- `docs/comments.md`
- `docs/variables.md`
- `docs/constants.md`
- `docs/strings.md`
- `docs/numeric-expressions.md`
- `docs/lists.md`
- `docs/commands.md`
- `docs/exit.md`
- `docs/functions.md`
- `docs/conditionals.md`
- `docs/repeat.md`
- `docs/while.md`
- `docs/modules.md`

## Tasks

- [x] Rewrite README as concise project entrypoint with linked sections.
- [x] Create per-topic docs with clear rules and examples.
- [x] Turn `docs/language.md` into a language index/overview.
- [x] Verify docs links and examples structurally.
- [x] Run focused verification.

## Evidence

- `cargo fmt --check` PASS.
- `cargo test` PASS: 74 tests.
- Markdown local link check PASS: 15 markdown files checked.
- `printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` PASS; final output included `Status: count is high enough`.

## Notes

- No language behavior changed.
- The docs split keeps the README short and moves detailed rules into focused topic files.
- Work has not been committed or pushed yet in this task.
