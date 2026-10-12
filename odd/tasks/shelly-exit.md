# Shelly exit

Feature: add `exit` for terminating the whole script with an OS exit code.

## Decisions

- `exit` terminates the whole Shelly program immediately.
- `exit` without a code is equivalent to `exit 0`.
- `exit 0` means success.
- `exit 1` means general failure.
- Other integer codes from `0` to `255` are allowed.
- `exit` can be used inside any block: top-level, `if`, `repeat`, `while`, and functions.
- `exit` exits the whole script, not just the current block or function.
- Exit codes may be integer literals, numeric expressions, variables, or constants.
- Exit codes must resolve to integer numeric values.
- Non-numeric, decimal, negative, greater-than-255, and malformed codes are errors.

## Examples

```ly
if $answer != "yes"
    shellout "Cancelled"
    exit 1
end

shellout "Done"
exit 0
```

## Tasks

- [x] Add failing tests for top-level and nested `exit`.
- [x] Implement process exit propagation.
- [x] Add invalid exit code tests.
- [x] Update README, language guide, example, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed before implementation with 7 failing exit tests: top-level `exit`, `exit 0`, nested `if`/`repeat`/`while`/function propagation, and invalid-code diagnostics were unimplemented. `exit 1` already returned status 1 only because unknown-command errors exit 1, so it was not sufficient behavior by itself.
- GREEN: `cargo fmt --check && cargo test && printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` passed after implementation. `cargo test` reported 74 passed, 0 failed.
- Sample run: `example-01.ly` completed the full showcase and ended with `Status: count is high enough`.
