# Shelly minimal purple docs

Feature: make the public documentation more minimal, purple-accented, and badge-driven.

## Goals

- Fetch and integrate the user's latest README image change before editing.
- Make README more minimal and visually polished.
- Add purple accents that work on GitHub Markdown.
- Add stats badges for releases, downloads, stars, and license/private status.
- Keep links to focused docs.
- Avoid changing language behavior.
- Commit and push when verified, per project preference.

## Tasks

- [x] Refresh from remote and inspect current README image.
- [x] Redesign README with minimal structure, purple accents, and badges.
- [x] Lightly adjust docs overview if needed.
- [x] Verify markdown links and Rust checks.
- [ ] Commit and push.

## Evidence

- Fetched and fast-forwarded `origin/main` to `410ada6 Update README.md`.
- `README.md` now keeps the user-added image, adds purple badges, purple dividers, a minimal intro, a short example, and a docs table.
- `docs/language.md` is now a minimal overview with purple dividers and the topic index.
- `cargo fmt --check` PASS.
- `cargo test` PASS: 74 tests.
- Markdown local link check PASS: 15 markdown files checked.
- `printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` PASS; final output included `Status: count is high enough`.

## Notes

- GitHub Markdown strips arbitrary CSS, so purple accents use GitHub-compatible image badges/dividers.
- The repository is private and has no license file yet, so the README uses a purple `license-private` badge instead of inventing a license.
