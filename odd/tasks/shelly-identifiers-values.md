# Shelly identifiers and literal values

Feature: implement identifier and value rules for variables and constants.

## Decisions

- Variables use `$name`.
- Constants use `$$NAME`.
- Variables are case-insensitive: `$name`, `$Name`, and `$NAME` refer to the same variable.
- Constants must be uppercase.
- Identifiers must start with an ASCII letter.
- After the first letter, identifiers may contain ASCII letters, numbers, or `_`.
- Identifiers cannot start with numbers, `_`, or other symbols.
- Identifiers cannot contain spaces.
- Values may be strings, numbers, booleans, or empty variables.
- Variables may be declared without a value using a bare declaration: `$name`.
- Constants must always have an initial value.

## Tasks

- [x] Add failing tests for constants, case-insensitive variables, numeric/boolean literals, and bare variable declarations.
- [x] Implement value parsing, variable normalization, constants, and identifier validation.
- [x] Update language documentation and examples.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed before implementation with 8 passed / 3 failed in `tests/cli_smoke.rs`: `identifiers_must_start_with_ascii_letter`, `values_support_numbers_booleans_and_constant_interpolation`, and `variables_are_case_insensitive_and_may_be_empty`.
- GREEN: `cargo fmt --check && cargo test` passed after implementation: 11 passed / 0 failed.
- SAMPLE: `cargo run --quiet -- example-01.shelly` printed `me llamo Agustin, y tengo 27 años.`.
