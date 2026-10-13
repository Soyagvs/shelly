# Shelly files glob command

Feature: add a Shelly-native way to collect filesystem paths into a list.

## Decisions

- Add `files "pattern" -> $items`.
- `files` stores a Shelly list of matching file paths in `$items`.
- Running `files` alone does not print or loop; it only assigns the list. Users can print it with `shellout` or iterate with `for`.
- Match paths relative to the current script file directory.
- Start with simple glob patterns useful for scripts, especially `*.ext` and `dir/*.ext`.
- Keep `for` unchanged; `files` only creates the list that `for` can iterate.

## Tasks

- [x] Implement `files "pattern" -> $var`.
- [x] Add check validation and CLI tests.
- [x] Document `files` in docs and README links where needed.
- [x] Verify formatting and tests.

## Evidence

- Added runtime support for `files "pattern" -> $items`.
- Added simple non-recursive glob matching:
  - `*.txt`
  - `dir/*.txt`
  - exact filenames like `README.md`
  - `*` within one path segment
- Added checker validation for valid and malformed `files` syntax.
- Added `docs/files.md` and linked every `docs/*.md` from root README.
- Verification passed:
  - `cargo fmt --check`
  - `cargo test files` with 6 tests passed
  - `cargo test` with 103 tests passed
  - README docs link check: 18 docs, 0 missing links
  - manual smoke printed `[a.txt, b.txt]` and iterated `a.txt`, `b.txt`
- Native Gentle review inspect remained blocked by `package-local-binary-missing`.
