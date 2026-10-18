# Shelly case blocks

Feature: add `case` / `when` / `else` control flow for exact value dispatch.

## Approved syntax

```shelly
case VALUE
    when VALUE
        ...
    when VALUE or VALUE or VALUE
        ...
    else
        ...
end
```

## Semantics

- Use `case`, not `switch`.
- `case VALUE` evaluates one value.
- `when VALUE` compares exact equality against the case value; alternatives use repeated `or`.
- `when VALUE or VALUE` matches if either value equals the case value.
- `else` is optional.
- First match wins.
- No fallthrough and no `break` needed.
- Blocks use `end` like other Shelly control-flow blocks.
- `when` conditions are values, not complex boolean conditions.

## Tasks

- [x] Explore current block parser/checker behavior.
- [x] Implement `case` runtime execution.
- [x] Implement checker validation and clear errors.
- [x] Update docs/examples/tests.
- [x] Verify formatting and tests.

## Evidence

- `cargo fmt --check` passed.
- `cargo test` passed: 143 tests, including case runtime/checker coverage.
- Added malformed-form coverage for missing case value, when outside case, duplicate else, when after else, empty when, missing end, no when, and obsolete comma-separated alternatives.
