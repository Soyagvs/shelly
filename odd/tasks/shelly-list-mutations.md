# Shelly list mutations

Feature: add basic list mutation operations.

## Decisions

- Lists can be mutated only through variable lists, not constants or temporary selector results.
- `append $list value` adds `value` to the end of `$list` (arguments are whitespace-separated).
- `prepend $list value` adds `value` to the beginning of `$list` (arguments are whitespace-separated).
- `remove $list[first]` removes the first item.
- `remove $list[last]` removes the last item.
- `remove $list[index]` removes one item by 1-based index.
- `$list[index] = value` replaces one item by 1-based index.
- `first` and `last` are valid mutation indexes.
- Mutation indexes are 1-based and must select exactly one item.
- Multi-indexes, ranges, full-copy selectors, and filters are not mutation targets yet.
- Out-of-range mutation indexes are errors.
- Mutating an empty list with `first` or `last` is an error.
- Values accepted by append/prepend/replacement match first-version list scalar elements: strings, numbers/numeric expressions, booleans, variables, and constants.
- Nested lists are not supported yet.

## Examples

```shelly
$names = ["Ana", "Luis"]
append $names "Mora"
prepend $names "Iris"
$names[2] = "Ada"
remove $names[last]
remove $names[first]
shellout $names[..]
```

## Tasks

- [x] Add failing tests for append/prepend/remove/replacement.
- [x] Implement basic list mutations.
- [x] Add negative tests for invalid mutation targets and indexes.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` failed after tests were added, before implementation: 61 passed, 3 failed (`list_mutations_append_prepend_remove_replace_and_persist`, `list_mutations_accept_scalar_values`, `list_mutation_errors_are_clear`) because `append` was unknown.
- GREEN: `cargo test` passed after implementation: 64 passed.
- Historical comma-form examples were repaired to current whitespace command syntax; the original evidence and implementation timeline are otherwise preserved.
- Format: `cargo fmt --check` passed.
- Example: `printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly` passed after updating `example-01.shelly`.

## Deferred list features

- Multi-index, range/full-copy selector, and filter mutation targets.
- Nested list elements.
- Additional list operations such as sort/contains/len/delete-by-value.
