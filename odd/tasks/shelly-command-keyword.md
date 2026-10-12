# Shelly command keyword

Feature: replace `run` with `command` for executing Linux/system shell commands.

## Decisions

- `command "..."` executes a system command through the operating system shell.
- `run "..."` is removed because Shelly is new and does not need backward compatibility yet.
- Historical note: `capture` initially kept its role here, but it was later removed. Current syntax is `command "..." -> $var`; see `odd/tasks/shelly-command-capture-arrow.md`.

## Tasks

- [x] Add failing tests for `command` and removed `run`.
- [x] Implement `command` and remove `run` from the runtime.
- [x] Update README, language guide, and ODD notes.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed before implementation with `command_streams_system_command_output` rejected as unknown syntax and `run_keyword_is_rejected` still succeeding; 11 passed, 2 failed.
- GREEN: `cargo fmt --check` passed after implementation.
- GREEN: `cargo test` passed after implementation; 13 integration tests passed.
- Manual demonstration: `cargo run --quiet -- <(printf 'command "printf demo-command"\n')` printed `demo-command`.
