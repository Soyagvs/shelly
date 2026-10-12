# Shelly functions

Feature: add function definitions, calls, parameters, defaults, local scope, and explicit globals.

## Decisions

- Functions start with `fn` and end with `end`.
- Functions are called by name.
- Definitions use commas between parameters: `fn person $name, $age`.
- Calls use commas between arguments: `person "Agustin", 27`.
- Parameters use `$` variable syntax.
- Parameters may have default values: `fn greet $name, $prefix = "Hola"`.
- Parameters with defaults must come after all required parameters.
- Function scope is local by default.
- Variables/constants from outside a function must be declared with `global` before use.
- `global` can declare multiple identifiers separated by commas:
  - `global $name, $age`
  - `global $$NAME, $$AGE`
- Initial `global` behavior is read-only for outer values.
- Functions do not return values in the first implementation.

## Tasks

- [x] Add failing tests for function definitions, calls, parameters, defaults, local scope, and globals.
- [x] Implement function parsing, registration, calls, scoped execution, and global reads.
- [x] Add negative tests/errors for missing `end`, duplicate functions, argument count errors, required-after-default parameters, and invalid global declarations.
- [x] Update README, language guide, and examples with clear itemized rules.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` after adding function tests failed as expected: `functions_call_with_parameters_defaults_local_scope_and_global_reads` failed on `line 4: unknown command or invalid syntax: fn person...`; `function_errors_cover_invalid_definitions_and_calls` failed because `missing end` was not implemented.
- GREEN: `cargo fmt --check && cargo test && cargo run --quiet -- examples/functions.ly` passed after implementation.
- Test result: `cargo test` reported 15 passed, 0 failed.
- Manual sample output:

```text
Hola Agustin age=27 from Shelly with outer
outer
```
