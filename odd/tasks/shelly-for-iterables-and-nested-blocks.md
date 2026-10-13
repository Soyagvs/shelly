# Shelly iterable for loops and nested blocks

Feature: make list iteration and list-condition behavior feel consistent across Shelly block constructs.

## Decisions

- `for $item in $$LIST` iterates constant lists.
- `for $item in $list[selector]` iterates list-producing selectors, ranges, and filters.
- Scalar selectors such as `$list[1]` are not iterable and produce a clear error.
- `if $list[filter]` and `while $list[filter]` use list truthiness: non-empty lists are true, empty lists are false.
- Scalar conditions keep existing truthiness rules.
- Nested `if`, `repeat`, `while`, and `for` blocks should work in every combination.
- Loop variables keep current `for` behavior: after a non-empty loop, the variable remains set to the last iterated value.

## Examples

```shelly
$$NAMES = ["Ana", "Luis"]
for $name in $$NAMES
    shellout $name
end
```

```shelly
$numbers = [1, 3, 5, 8]
for $number in $numbers[>3]
    shellout $number
end

if $numbers[>10]
    shellout "has big numbers"
else
    shellout "no big numbers"
end
```

## Tasks

- [x] Add failing tests for constant-list iteration and list selector/filter iteration.
- [x] Add failing tests for list-filter truthiness in `if`/`while`.
- [x] Add failing tests for nested `for`/`if`/`while` combinations.
- [x] Implement list-preserving iterable and condition evaluation.
- [x] Update docs/examples and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test for_list_iterates_constant_lists_selectors_ranges_and_filters -- --nocapture` failed because `for` still rejected constant lists (`for syntax is for $item in $list`).
- RED: `cargo test list_filters_are_truthy_when_non_empty_and_falsey_when_empty_in_blocks -- --nocapture` failed because condition parsing treated `>` inside `$numbers[>3]` as a top-level comparison and reported `malformed list index syntax`.
- BASELINE: `cargo test nested_control_blocks_work_in_combination -- --nocapture` passed before the evaluator change, confirming nested block depth already handled the requested `for`/`if`/`while` combination.
- GREEN: targeted tests for constant/selector/filter iteration, list-filter truthiness, and nested block combinations passed after implementing list-preserving evaluation.
- GREEN: `cargo fmt --check && cargo test && cargo run --quiet -- examples/for-lists.shelly && printf 'yes\\nblue\\n' | cargo run --quiet -- example-01.shelly | grep -E 'For loop|Filtered for loop|No numbers above ten'` passed; full suite reported 80 tests passed.
