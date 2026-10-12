# Shelly command arrow capture

Feature: replace standalone `capture` with `command "..." -> $var`.

## Decisions

- `command "..."` executes a system command and streams output normally.
- `command "..." -> $var` executes a system command and captures stdout into `$var`.
- Captured stdout trims trailing newlines.
- Captured command stdout is not printed automatically.
- Command failures remain errors.
- Capture destination must be a variable, not a constant.
- `capture $var, "..."` is removed/rejected because `command ... -> $var` is the official form.

## Tasks

- [x] Add failing tests for `command "..." -> $var` and removed `capture`.
- [x] Implement arrow capture in `command`.
- [x] Update examples and documentation to remove `capture` as a supported feature.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed with 6 new failing CLI smoke tests before implementation: arrow capture stdout/no direct print, newline trimming, interpolation, old `capture` rejection, constant destination rejection, and malformed destination rejection.
- GREEN: `cargo fmt --check && cargo test` passed; CLI smoke suite reported 40 passed.
- Example: `printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` passed and printed the updated command-capture flow, including `Running from /home/soyagvs/proyectos/personal/shelly`.
