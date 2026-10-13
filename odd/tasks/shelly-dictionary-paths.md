# Shelly dictionary chained paths

Feature: complete dictionary access with chained reads, writes, and removal.

## Approved syntax

```shelly
$user = {
    pais: {
        argentina: {
            city: "Buenos Aires"
        }
    }
}

shellout $user.pais.argentina.city
$user.pais.argentina.city = "Córdoba"
remove $user.pais.argentina.city
```

## Intended semantics

- Chained dot access reads nested dictionary keys.
- Chained bracket access reads nested dictionary keys.
- Mixed access works: `$user.pais["argentina"].city`.
- Chained assignment mutates nested dictionary keys.
- `remove $dict.path` removes a dictionary key, analogous to list remove.
- Missing intermediate keys are errors.
- Mutating through a non-dictionary value is an error.
- Removing a missing key should be a clear error.

## Tasks

- [x] Explore current dictionary access/mutation/remove parsing.
- [x] Implement chained dictionary read access.
- [x] Implement chained dictionary mutation.
- [x] Implement dictionary key removal.
- [x] Update checker validation and errors.
- [x] Update docs/examples/tests.
- [x] Verify formatting and tests.

## Evidence

- Added CLI smoke tests for chained read, mixed bracket/dot access, chained mutation, interpolation, remove, missing keys, non-dictionary traversal, constant mutation rejection, and list/file remove preservation.
- `cargo fmt --check` passed.
- `cargo test` passed.
