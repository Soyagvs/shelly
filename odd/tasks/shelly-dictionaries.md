# Shelly dictionaries

Feature: add dictionary/map values for structured script data.

## Approved direction

Add dictionaries before JSON so Shelly can model structured data.

## Implemented syntax

```shelly
$user = {
    name: "Ada"
    age: 32
    admin: true
    "first-name": "Ada"
}

shellout $user.name
$user.name = "Grace"

$key = "first-name"
shellout $user[$key]
shellout $user["first-name"]
```

## Implemented semantics

- Dictionaries are first-class values.
- Simple keys may be unquoted identifiers.
- Non-identifier keys use string keys.
- Dot access reads/writes simple keys.
- Bracket access reads dynamic or quoted keys.
- Missing keys are runtime errors.
- Dictionary values may be scalars, lists, or dictionaries.
- `shellout $dict` renders keys deterministically.
- String interpolation supports `$dict.key` and `$dict[$key]`.
- Empty dictionaries are false; non-empty dictionaries are true.
- `contains $dict "key"` checks dictionary keys.

## Limitations

- Lists remain scalar-only; nested lists and dictionaries inside lists are rejected.
- Dictionary access composes through multiple levels in one expression, including mixed dot/bracket paths such as `$user.profile[$key].name`.

## Tasks

- [x] Explore value representation, assignment, interpolation, selectors, docs, and tests.
- [x] Implement dictionary literal parsing.
- [x] Implement dictionary property access.
- [x] Implement dictionary property assignment/mutation.
- [x] Update checker validation and errors.
- [x] Update docs/examples/tests.
- [x] Verify formatting and tests.

## Evidence

- `git diff --check` passed.
- `cargo fmt --check` passed.
- `cargo test` passed with 146 tests and no Rust warnings observed.
- README local markdown/docs-link check passed.
- Manual smoke passed for dictionary literal, dot access, dynamic bracket access, mutation, `contains $dict "key"`, interpolation, and `shellout $dict`.
