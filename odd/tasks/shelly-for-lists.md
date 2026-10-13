# Shelly for loops over lists

Feature: add `for` loops for iterating list values.

## Decisions

- `for $item in $list` iterates over each element of a list.
- The loop body runs once per list element, in list order.
- The loop variable is assigned for each iteration and remains available after the loop with the last iterated value.
- Empty lists run zero iterations.
- `for` uses `end`, like `fn`, `if`, `repeat`, and `while`.
- The iterable expression must currently be a list variable.
- Iterating a non-list value is an error with a clear diagnostic.
- Missing loop variable, missing `in`, missing iterable, and missing `end` are clear errors.

## Examples

```shelly
$names = ["Ana", "Luis", "Mora"]

for $name in $names
    shellout "Hello $name"
end
```

```shelly
$empty = []

for $item in $empty
    shellout "This will not print"
end
```

## Tasks

- [x] Add failing tests for `for $item in $list` loops.
- [x] Implement `for` list execution.
- [x] Add negative tests for malformed `for` syntax, non-list iterables, and missing `end`.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test for_list -- --nocapture` failed with 3 new for-list tests because `for` was unknown (`unknown command or invalid syntax: for $name in $names`) and malformed `for` diagnostics were not implemented yet.
- GREEN: `cargo fmt --check && cargo test for_list -- --nocapture` passed after implementing `for` execution.
- GREEN: `cargo fmt --check && cargo test && cargo run --quiet -- examples/for-lists.shelly && printf 'yes\\nblue\\n' | cargo run --quiet -- example-01.shelly | grep 'For loop'` passed; full suite reported 77 tests passed and both examples printed the expected for-loop output.
