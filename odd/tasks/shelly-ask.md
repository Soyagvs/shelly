# Shelly ask command

Feature: add `ask` as a prompt-and-read convenience command.

## Decisions

- `ask` requires a prompt argument followed by a capture destination.
- Current syntax: `ask "Question?" -> $variable`.
- The prompt is printed before reading one line into the destination variable.
- `ask` is not a print-only command; use `shellout` for printing without input.
- The destination must be a variable (`$name`), not a string or constant.
- The input value is stored without the trailing line ending.

## Tasks

- [x] Add failing tests for `ask` success and invalid argument forms.
- [x] Implement `ask` command.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- Historical RED: before `ask` was implemented, `cargo test` failed with `ask_prints_prompt_reads_stdin_line_and_stores_variable` and `ask_rejects_invalid_argument_forms`. The earlier comma-form diagnostic is obsolete; current syntax is `ask "Name?" -> $name`.
- GREEN: `cargo fmt --check` passed.
- GREEN: `cargo test` passed with 19 tests.
- Manual example: `printf 'yes\n' | cargo run --quiet -- examples/ask-if.shelly` printed `Continue? continuing`.
