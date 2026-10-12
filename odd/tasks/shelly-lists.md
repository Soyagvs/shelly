# Shelly lists

Feature: add Shelly's single collection type: list.

## Decisions

- Shelly has one collection type: `list`.
- Lists replace separate arrays, tuples, vectors, slices, etc. at the language level.
- Empty list literal: `[]`.
- List literal with values: `["Ana", "Luis", "Mora"]`.
- Elements initially supported:
  - strings
  - numbers
  - booleans
  - variables
  - constants
- List values can be assigned to variables.
- List indexing uses 1-based indexes to match `repeat ... -> $i`.
- Access syntax: `$list[1]`.
- List indexing works in `shellout` and string interpolation.
- Out-of-range access is an error with a clear diagnostic.
- Invalid indexes are errors.
- No list mutation operations yet: no push, pop, remove, delete, sort, contains, or len in this first version.

## Examples

```shelly
$empty = []
$names = ["Ana", "Luis", "Mora"]
$numbers = [1, 2, 3]
$mixed = ["Ana", 3, true]

shellout $names[1]
shellout "Second: $names[2]"
```

## Tasks

- [x] Add failing tests for list literals and indexing.
- [x] Implement list value parsing/storage and 1-based indexing.
- [x] Add negative tests for invalid and out-of-range indexing.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED `cargo test`: failed as expected with new list tests (`lists_accept_empty_assignment_and_index_values`, `lists_accept_variable_and_constant_elements_and_interpolation_indexing`, `list_indexing_errors_are_clear`) before implementation.
- GREEN `cargo fmt --check`: passed.
- GREEN `cargo test`: passed, 58 tests.
- GREEN `printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly`: passed and printed the list demo plus the existing sample flow.
