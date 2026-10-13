# Shelly standard helpers

Feature: add standard helper commands for text, lists, conversions, paths, time, and random values.

## Decisions

- Add text helpers:
  - `trim $text -> $result`
  - `lower $text -> $result`
  - `upper $text -> $result`
  - `replace $text "old" "new" -> $result`
  - `split $text "," -> $items`
  - `startsWith $text "prefix" -> $ok`
  - `endsWith $text "suffix" -> $ok`
- Add list helpers:
  - `len $items -> $count`
  - `join $items ", " -> $text`
  - `sort $items -> $sorted`
  - `reverse $items -> $reversed`
  - `unique $items -> $unique`
- Add conversion/validation helpers:
  - `toNumber $text -> $number`
  - `toString $value -> $text`
  - `isEmpty $value -> $ok`
  - `isNumber $value -> $ok`
  - `isText $value -> $ok`
  - `isList $value -> $ok`
- Add path helpers:
  - `basename $path -> $name`
  - `dirname $path -> $dir`
  - `extension $path -> $ext`
  - `withoutExtension $path -> $name`
- Add time helpers:
  - `now -> $timestamp`
  - `today -> $date`
- Add random helpers:
  - `random -> $value`
  - `randomInt min max -> $number`
- Every helper must be documented with at least one example.
- Root README must link every `docs/*.md` file.

## Tasks

- [x] Implement helper commands and checker validation.
- [x] Add CLI smoke tests for each helper group.
- [x] Document helpers with examples.
- [x] Verify formatting, tests, and docs links.

## Evidence

- Added `docs/helpers.md` with examples for every helper.
- Added runtime support for all planned helper groups:
  - text helpers
  - list helpers
  - conversion/validation helpers
  - path string helpers
  - time helpers
  - random helpers
- Added checker validation for helper syntax.
- Added CLI smoke tests, including deterministic `randomInt 5 5 -> $n`.
- Verification passed:
  - `cargo fmt --check`
  - `cargo test standard_helper`
  - `cargo test` with 119 tests passed
  - manual smoke covering representative helpers from every group passed
  - README local markdown/docs links passed
- Native Gentle review inspect remained blocked by `package-local-binary-missing`.
