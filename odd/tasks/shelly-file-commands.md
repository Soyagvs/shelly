# Shelly file commands

Feature: add first-class file commands for common script automation.

## Decisions

- Add path commands:
  - `create "path"`
  - `copy "from" "to"`
  - `move "from" "to"`
  - `rename "from" "new-name"`
  - `remove "path"`
  - `read "file" -> $text`
  - `write "file" $text`
- Resolve relative paths from the current Shelly script directory, consistent with `exists`, `files`, `import`, and `include`.
- `./` is supported and points to the current Shelly script directory.
- `create` behavior:
  - path ending in `/` creates a directory.
  - path not ending in `/` creates an empty file.
  - creates parent directories automatically.
  - does not fail if the file/directory already exists.
  - does not overwrite existing file content.
- `copy` copies files or directories recursively, creates parent directories, and fails if destination exists.
- `move` moves files or directories, creates parent directories, and fails if destination exists.
- `rename "from" "new-name"` renames within the same directory; `new-name` must be a plain file/directory name, not a path.
- `remove` deletes files; deletes directories recursively only when the path ends with `/`; missing paths do not fail.
- `read` reads UTF-8 text from a file into a variable.
- `write` writes text to a file, creates parents, and overwrites content.
- Do not add `appendFile`, `size`, `isFile`, or `isDir` yet.

## Tasks

- [x] Implement file commands and path helpers.
- [x] Add checker validation and CLI tests.
- [x] Document file commands.
- [x] Verify formatting and tests.

## Evidence

- Added runtime commands:
  - `create "path"`
  - `copy "from" "to"`
  - `move "from" "to"`
  - `rename "from" "new-name"`
  - `remove "path"`
  - `read "file" -> $text`
  - `write "file" $text`
- Added checker validation for valid and malformed file command forms.
- Added tests for:
  - nested `create` with `./` script-relative paths
  - create preserving existing file contents
  - file and directory copy
  - destination-exists failures
  - move and rename behavior
  - remove file/missing path/directory with trailing slash
  - directory remove without trailing slash safety error
  - read/write behavior
  - invalid check syntax
  - `./` import/include coverage
- Updated docs:
  - `docs/files.md`
  - `docs/commands.md`
  - `docs/language.md`
  - `README.md`
- Verification passed:
  - `cargo fmt --check`
  - `cargo test file_commands` with 4 targeted tests passed
  - `cargo test` with 110 tests passed
  - README links all 18 `docs/*.md` files
  - manual smoke used `create`, `write`, `read`, `copy`, `rename`, `move`, `remove`, and `./` paths successfully
- Native Gentle review inspect remained blocked by `package-local-binary-missing`.
