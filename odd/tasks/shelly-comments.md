# Shelly comments

Feature: replace old loose comment handling with explicit line and block comments.

## Decisions

- Existing old comment behavior is removed.
- A line comment starts and ends on the same line with `#`:
  - `# comment #`
- A block comment starts with `#` and ends with a later `#`.
- A block comment may span multiple lines:
  - `#`
  - `comment body`
  - `#`
- Comment markers are only recognized outside strings.
- Unterminated comments are errors.

## Tasks

- [x] Add failing tests for explicit line comments, block comments, old comment rejection, and unterminated comments.
- [x] Implement explicit comment parsing/removal.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed after adding tests with 23 passed / 3 failed. Failing cases: `explicit_block_comment_is_ignored`, `old_style_comment_without_closing_marker_is_rejected_when_never_closed`, and `unterminated_block_comment_reports_start_line`.
- GREEN: `cargo fmt --check` passed.
- GREEN: `cargo test` passed with 26 passed / 0 failed.
- Sample: `printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` passed and printed the updated example flow ending with `Status: count is high enough`.
