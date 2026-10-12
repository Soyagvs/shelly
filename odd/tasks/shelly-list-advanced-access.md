# Shelly advanced list access and filters

Feature: add advanced list selectors and filters.

## Decisions

- Shelly lists stay 1-based.
- Single item selectors:
  - `$list[1]` returns the first item.
  - `$list[first]` returns the first item.
  - `$list[last]` returns the last item.
- Multi-selectors return a new list:
  - `$list[1,3]` returns items 1 and 3.
  - `$list[1..3]` returns an inclusive slice from 1 to 3.
  - `$list[..3]` returns from first item through item 3.
  - `$list[3..]` returns from item 3 through last item.
  - `$list[..]` returns a copy of the whole list.
  - `$list[first..last]` is also a full-list slice.
- Ranges are inclusive.
- Reverse ranges like `$list[3..1]` are errors for now.
- Out-of-range selectors are errors.
- Filters return a new list.
- Numeric filters support `==`, `!=`, `>`, `>=`, `<`, `<=`:
  - `$numbers[>5]`
  - `$numbers[<=10]`
- String filters support `==` and `!=` only:
  - `$names[=="Ana"]`
  - `$names[!="Ana"]`
- Boolean filters support `==` and `!=`:
  - `$flags[==true]`
- Mixed lists may use `==` and `!=` by exact value matching.
- Ordered filters (`>`, `>=`, `<`, `<=`) require numeric list items; non-numeric items are errors.
- String ordered comparisons like `$names[>"Ana"]` are not supported yet.
- Existing no-mutation rule remains: no append/prepend/remove/index assignment in this task.

## Examples

```ly
$names = ["Ana", "Luis", "Mora"]
$numbers = [1, 3, 5, 8, 10]
$mixed = ["Ana", 3, true]

shellout $names[first]
shellout $names[last]
shellout $names[1,3]
shellout $names[1..2]
shellout $names[..]
shellout $numbers[>5]
shellout $names[=="Ana"]
shellout $mixed[!=3]
```

## Tasks

- [x] Add failing tests for advanced selectors.
- [x] Implement advanced selectors and filters.
- [x] Add negative tests for invalid selectors and filters.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test advanced_list -- --nocapture` failed before implementation: `first`/filter selectors reported `list index must be numeric`; reverse range reported an out-of-range index instead of the required reverse-range error.
- Focused GREEN: `cargo test advanced_list -- --nocapture` passed after implementation: 3 passed.
- Full GREEN: `cargo fmt --check && cargo test` passed after formatting: 61 tests passed.
- Example run: `printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` passed and printed advanced selector/filter examples such as `[Ana, Mora]`, `[8, 10]`, `[Ana]`, and `[true, true]`.

## Intentionally deferred

- List mutation remains unsupported: no append/prepend/remove/delete/sort/contains/len operations and no index assignment.
- Ordered string filters remain unsupported and error clearly.
- Reverse ranges remain unsupported and error clearly.
