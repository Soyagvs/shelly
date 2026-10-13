# Shelly explicit function calls

Feature: require `call` when invoking Shelly functions.

## Decisions

- Function definitions stay `fn name ... end`.
- Function calls use `call name args`.
- Returning function calls use `call name args -> $result`.
- Bare function invocation without `call` is rejected because it is visually ambiguous with built-in commands.
- Existing docs and examples should use `call` everywhere.

## Examples

```shelly
fn greet $name
    shellout "Hello $name"
end

call greet "Ada"
```

```shelly
fn make_greeting $name
    return "Hello $name"
end

call make_greeting "Ada" -> $message
shellout $message
```

## Tasks

- [x] Add failing tests for explicit `call` invocation and bare-call rejection.
- [x] Implement `call` dispatch and reject bare function calls.
- [x] Update docs/examples and task evidence.
- [x] Run verification.

## Evidence

- RED: targeted function tests failed because `call person "Agustin", 27` and `call label "Ada" -> $message` were unknown, while bare `greet "Ada"` still succeeded.
- GREEN: targeted function tests passed after adding `call` dispatch and rejecting bare function invocation.
- GREEN: `cargo fmt --check && cargo test && printf 'yes\\nblue\\n' | cargo run --quiet -- example-01.shelly` passed; full suite reported 84 tests passed and the showcase completed with `Status: count is high enough`.
