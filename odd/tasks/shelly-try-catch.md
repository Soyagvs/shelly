# Shelly try/catch error handling

Feature: add simple recoverable error handling with `try` / `catch`.

## Decisions

- Add `try` / `catch` / `end` blocks.
- Add optional error capture: `try -> $error`.
- `catch` is required exactly once for every `try` in the first version.
- `try` catches ordinary runtime errors such as failed `command`, missing `include`, invalid calls, or assignment/evaluation errors.
- `try` must not catch control signals: `exit`, `break`, `continue`, or function `return`.
- No `finally` in the first version.
- Blocks do not create a new scope, consistent with existing `if` / loop behavior.

## Tasks

- [x] Implement runtime parsing/execution for `try` / `catch`.
- [x] Implement checker validation for `try` / `catch`.
- [x] Add CLI tests for success, caught errors, captured errors, nested try, and non-captured control signals.
- [x] Document error handling.
- [x] Verify with formatting and tests.

## Evidence

- Added runtime support for:
  - `try` / `catch` / `end`
  - `try -> $error` / `catch` / `end`
- Added checker validation for:
  - missing `catch`
  - duplicate `catch`
  - `catch` outside `try`
  - malformed `try ->` destination
- Added tests for:
  - try success skips catch
  - ordinary errors enter catch
  - captured error message
  - nested try
  - `exit`, `break`, `continue`, and function `return` not being caught
  - check-mode malformed try/catch errors
- Added docs:
  - `docs/errors.md`
  - README docs table link
  - `docs/language.md` reference link
  - notes in `docs/commands.md` and `docs/conditionals.md`
- Verification passed:
  - `cargo fmt --check`
  - `cargo test try` with 4 try/catch tests passed
  - `cargo test` with 98 tests passed
  - manual `try -> $error` smoke caught `command "false"`
  - manual `exit 7` inside `try` was not caught and exited with status 7
  - README links every `docs/*.md` file: 17 docs found, 0 missing links
