# Shelly script arguments

Feature: let Shelly scripts read command-line arguments passed after the script file.

## Approved syntax

```shelly
arg 1 -> $name
hasArg "--dry-run" -> $dryRun
args -> $all
```

## Semantics

- `arg N -> $var` reads the Nth script argument, starting at 1.
- Missing positional arguments become an empty string.
- `hasArg VALUE -> $var` stores `true` when an argument exactly equals VALUE, otherwise `false`.
- `args -> $var` stores all script arguments as a list.
- Script arguments exclude the `shelly` executable and the script path.

## Tasks

- [x] Explore CLI execution path and checker validation.
- [x] Implement runtime storage for script arguments.
- [x] Implement `arg`, `hasArg`, and `args` commands.
- [x] Add checker validation and errors.
- [x] Update docs/examples/tests.
- [x] Verify formatting and tests.

## Evidence

- Added CLI smoke tests for positional args, missing args, `hasArg` true/false, `args` iteration, checker acceptance, checker rejection, and runtime invalid expressions.
- `cargo fmt --check` and `cargo test` passed on 2026-10-13.
