# Shelly numeric expressions

Feature: add numeric expressions for assignments and value evaluation.

## Decisions

- Numeric expressions support integers and floating-point numbers.
- Operators: `+`, `-`, `*`, `/`.
- Normal precedence applies: `*` and `/` before `+` and `-`.
- Parentheses are supported.
- Negative numbers are supported.
- Variables and constants can participate in numeric expressions.
- Division by zero is an error with a clear diagnostic.
- Strings and booleans are not valid numeric operands.
- Malformed expressions fail clearly.
- Results print cleanly: integer-looking results as `3`, decimal results as `2.5`.

## Tasks

- [x] Add failing tests for numeric expressions, precedence, parentheses, floats, negatives, variables/constants, and division by zero.
- [x] Implement numeric expression parser/evaluator.
- [x] Add negative tests for strings/booleans and malformed expressions.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed before implementation with the new numeric tests: precedence/parentheses/floats/negatives/references failed with `expected string, number, or boolean value`; division by zero did not report `division by zero`; non-numeric operand test did not report `numeric operand`.
- GREEN: `cargo fmt --check && cargo test` passed after implementation; `cargo test` reported 35 passed.
- Example: `printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` passed and printed arithmetic output including `count=7 half=4 temperature=12.75 negative=-6 total=127.5`.
