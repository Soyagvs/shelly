# Shelly JSON helpers and nested values

Feature: complete structured data support with JSON helpers, dictionary helpers, and nested list/dictionary values.

## Approved scope

Implement:

1. JSON parse/stringify.
2. Dictionary helpers `keys` and `values`.
3. Lists of dictionaries.
4. Lists of lists.

## Implemented syntax

```shelly
read "package.json" -> $text
jsonParse $text -> $pkg
shellout $pkg.name

jsonStringify $pkg -> $json
write "out.json" $json

keys $pkg -> $keys
values $pkg -> $values
isDict $pkg -> $ok

$users = [
    { name: "Ada", role: "admin" },
    { name: "Luis", role: "user" }
]

shellout $users[1].name

$matrix = [[1, 2], [3, 4]]
shellout $matrix[2][1]
```

## Implemented semantics

- List values now store nested `Value` items.
- Lists can contain scalars, dictionaries, and lists.
- Nested access chains work across lists and dictionaries:
  - `$users[1].name`
  - `$matrix[2][1]`
  - `$data.users[1].name`
  - interpolation such as `"name=$users[1].name"`
- JSON helpers:
  - `jsonParse $text -> $value`
  - `jsonStringify $value -> $text`
- JSON mapping:
  - objects -> dictionaries
  - arrays -> lists
  - strings/numbers/booleans -> scalar values
  - `null` -> empty scalar string
  - empty scalar string -> `null` when stringified
- Dictionary helpers:
  - `keys $dict -> $keys`
  - `values $dict -> $values`
  - `isDict $value -> $ok`
- `len` works for strings, lists, and dictionaries.
- Scalar-list helpers remain available for scalar lists.

## Limitations documented

- `join`, `sort`, `reverse`, `unique`, filters, and list mutations are scalar-list features and reject nested list/dictionary items clearly.
- JSON null and empty Shelly strings share the same representation in this version.

## Tasks

- [x] Explore value/list representation and helper architecture.
- [x] Refactor lists to store nested `Value` safely.
- [x] Implement list-of-dicts/list-of-lists indexing and rendering.
- [x] Implement JSON parse/stringify.
- [x] Implement `keys` and `values` helpers.
- [x] Update checker validation and errors.
- [x] Update docs/examples/tests.
- [x] Verify formatting and tests.

## Evidence

- `git diff --check` passed.
- `cargo fmt --check` passed.
- `cargo test` passed with 154 tests and no Rust warnings observed.
- README docs-link check passed: 23 docs, 0 missing.
- Manual smoke passed for `jsonParse`, `jsonStringify`, `keys`, `values`, list of dictionaries, list of lists, nested access, and nested interpolation.
