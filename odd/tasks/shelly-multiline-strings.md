# Shelly multiline strings

Feature: add explicit multiline string literals for commands that accept text values.

## Decisions

- Double-quoted strings remain line-local:
  - `shellout "hello"`
  - one instruction per line still applies for `"..."`.
- Multiline strings use triple backticks:
  - opening delimiter: ```` ``` ````
  - closing delimiter: ```` ``` ````
- Initial target use case:
  - `shellout ``` ... ````, so ASCII banners and long output blocks can be written naturally.
- The multiline literal preserves internal newlines and indentation.
- Backtick multiline strings are language syntax, not Markdown.

## Tasks

- [x] Explore current string parsing and checker behavior.
- [x] Implement triple-backtick multiline parsing.
- [x] Add runtime/checker errors for unterminated multiline strings.
- [x] Update docs/examples/tests.
- [x] Verify formatting and tests.

## Evidence

- Added triple-backtick multiline string preprocessing before runtime/checker parsing.
- Supported syntax includes:
  - `shellout ``` ... ``` `
  - assignment from multiline strings
  - `write` and `append` with multiline strings
  - function call arguments with multiline strings
- Content inside multiline strings is not parsed as Shelly statements, comments, block starts, `end`, or `}`.
- Updated docs/examples:
  - `README.md`
  - `docs/strings.md`
  - `docs/commands.md`
  - `docs/language.md`
  - `example-01.shelly`
- Added tests for shellout, assignment, write/append, function call args, unterminated strings, and block-marker/comment content.
- Verification passed in worker:
  - `cargo fmt --check`
  - `cargo test` with 129 tests passed
