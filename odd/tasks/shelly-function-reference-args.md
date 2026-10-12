# Shelly function reference arguments

Feature: allow variables and constants as function call arguments.

## Decisions

- Function calls should accept literal values and references.
- Valid function arguments include strings, numbers, booleans, variables, and constants.
- Example: `greet $name, $$APP, 27, true`.
- Variable/constant references are evaluated at call time and passed as local parameter values.
- Existing literal argument behavior must keep working.

## Tasks

- [x] Add failing tests for function calls with `$variable` and `$$CONSTANT` arguments.
- [x] Implement reference argument evaluation in function calls.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed before implementation with `function_call_arguments_accept_variable_and_constant_references` and `function_call_reference_arguments_fail_when_undefined`; observed errors were `line 7: expected string, number, or boolean value` and `line 3: expected string, number, or boolean value`.
- GREEN: `cargo fmt --check && cargo test && printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly` passed after implementation.
- Sample output included:

```text
Welcome to Shelly, Agustin
Empty variable value: ''
Do you want to continue? Great, continuing
What is your favorite color?
Favorite color: blue
Command output: hello from Linux
Running from /home/soyagvs/proyectos/personal/shelly
Hello Agustin from Shelly; favorite color is blue
Hi Guest from Shelly; favorite color is blue
Status: count is high enough
```
