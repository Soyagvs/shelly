# Shelly repeat loops

Feature: add `repeat` loops for simple counted repetition.

## Decisions

- `repeat <numeric-expression>` repeats a block a fixed number of times.
- `repeat <numeric-expression> -> $name` optionally binds the current iteration number to a variable.
- The iteration variable name can be any valid Shelly variable identifier.
- The iteration variable starts at `1`.
- The iteration variable is local to the `repeat` block and should not overwrite an outer variable with the same name.
- `repeat 0` executes nothing.
- Negative counts are errors.
- Decimal counts are errors; repeat counts must be integer-looking numeric values.
- The count expression can use numeric expressions, variables, and constants.
- `repeat` uses `end`, like `fn` and `if`.
- The repeat block itself does not create a general new scope for normal variables; only the optional iteration variable is protected/local.

## Examples

```shelly
repeat 3
    shellout "Hola"
end

repeat 3 -> $turn
    shellout "Vuelta $turn"
end

$count = 0
repeat 3
    $count = $count + 1
end
shellout "count=$count"
```

## Tasks

- [x] Add failing tests for repeat loops and index binding.
- [x] Implement repeat execution.
- [x] Add negative tests for invalid counts and invalid index targets.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` after adding tests failed as expected: 8 repeat tests failed with `unknown command or invalid syntax: repeat ...`; existing 40 tests passed.
- GREEN: `cargo fmt --check` passed.
- GREEN: `cargo test` passed: 48 tests passed.
- Example: `printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly` passed and printed repeat turns `1`, `2`, `3` plus `Repeat visits counted: 2`.
