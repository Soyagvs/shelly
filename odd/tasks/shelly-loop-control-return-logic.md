# Shelly loop control, function returns, and logic

Feature: add the next core language controls: `break`, `continue`, function `return`, and logical conditions.

## Decisions

- `break` exits the nearest enclosing `repeat`, `while`, or `for` loop.
- `continue` skips to the next iteration of the nearest enclosing loop.
- `break` and `continue` outside a loop are clear errors.
- `return <value>` exits the current function with a scalar value.
- Function return capture uses Shelly's result-binding convention: `name args -> $result`.
- Calling a returning function without `->` is allowed and discards the returned value.
- `return` outside a function is a clear error.
- Conditions support `and`, `or`, and `not` with simple precedence: `not` > `and` > `or`.
- Existing comparisons and truthiness remain unchanged.
- Parenthesized condition expressions remain unsupported for now.

## Examples

```shelly
for $name in $names
    if $name == "Mora"
        break
    end
    shellout $name
end
```

```shelly
fn label $name
    return "hello $name"
end

label "Ada" -> $message
shellout $message
```

```shelly
if not $disabled and $count > 0
    shellout "ready"
end
```

## Tasks

- [x] Add failing tests for `break` and `continue` in all loop kinds.
- [x] Add failing tests for function `return` and `->` capture.
- [x] Add failing tests for `and`, `or`, and `not` conditions.
- [x] Implement control-flow signals and function return capture.
- [x] Implement logical condition parsing.
- [x] Update docs/examples and task evidence.
- [x] Run verification.

## Evidence

- RED: targeted tests failed before implementation: `continue` was unknown in loops, `label "Ada" -> $message` was parsed as malformed function arguments, and logical conditions still returned `unsupported complex condition`.
- GREEN: targeted tests passed after implementation: `break_and_continue_control_loop_iteration`, `function_return_can_be_captured_or_discarded`, and `logical_condition_operators_use_expected_precedence`.
- GREEN: `cargo fmt --check && cargo test && printf 'yes\\nblue\\n' | cargo run --quiet -- example-01.shelly` passed; full suite reported 84 tests passed and the showcase completed with `Status: count is high enough`.
