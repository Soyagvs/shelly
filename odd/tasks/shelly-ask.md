# Shelly ask command

Feature: add `ask` as a prompt-and-read convenience command.

## Decisions

- `ask` always requires exactly two comma-separated arguments.
- Syntax: `ask "Question?", $variable`.
- First argument is the prompt message to print.
- Second argument is the variable that stores one line of user input.
- `ask` is not a print-only command; use `shellout` for printing without input.
- The destination must be a variable (`$name`), not a string or constant.
- The input value is stored without the trailing line ending.

## Tasks

- [x] Add failing tests for `ask` success and invalid argument forms.
- [x] Implement `ask` command.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed before implementation with `ask_prints_prompt_reads_stdin_line_and_stores_variable` and `ask_rejects_invalid_argument_forms`; observed `line 2: unknown command or invalid syntax: ask "Name? ", $name` and missing `ask requires exactly two arguments` diagnostics.
- GREEN: `cargo fmt --check` passed.
- GREEN: `cargo test` passed with 19 tests.
- Manual example: `printf 'yes\n' | cargo run --quiet -- examples/ask-if.ly` printed `Continue? continuing`.
