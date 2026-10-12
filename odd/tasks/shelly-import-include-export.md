# Shelly import/include/export

Feature: link Shelly files with explicit public imports and full-file includes.

## Decisions

- `import "file.ly"` loads only exported symbols from the target file.
- `include "file.ly"` executes the whole target file.
- `export` marks public symbols inside a file.
- Initial exported symbols are constants and functions only.
- Exported constants use `export $$NAME = value`.
- Exported functions use `export fn name ... end`.
- Normal variables are not exportable initially.
- `import` does not execute non-exported top-level code from the imported file.
- `include` runs in the current scope, like copying the file contents at that point.
- Import/include paths are relative to the file that declares them.
- Cycles fail with a line-numbered error.
- Duplicate imports are ignored after the first successful import.

## Tasks

- [x] Add failing tests for importing exported constants/functions and not importing private symbols.
- [x] Add failing tests for include executing a full file in current scope.
- [x] Implement path-relative import/include/export with cycle protection.
- [x] Add negative tests for invalid exports, missing files, duplicate imports, and cycles.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

RED before implementation:

- `cargo test`
- Result: failed as expected.
- Observed failures: `import_loads_exported_constant`, `import_loads_exported_function`, `import_keeps_private_symbols_unavailable`, `import_does_not_run_non_exported_top_level_side_effects`, and `include_runs_full_file_in_current_scope` failed because `import`/`include` were unknown commands.

GREEN after implementation:

- `cargo fmt --check && cargo test && printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly`
- Result: passed.
- Test result: 32 passed, 0 failed.
- Sample output included:
  - `Welcome to Shelly, Agustin`
  - `Favorite color: blue`
  - `Hello Agustin from Shelly; favorite color is blue`
  - `Status: count is high enough`
