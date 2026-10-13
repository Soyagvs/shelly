# Shelly extra file helper commands

Feature: extend Shelly file automation with file type checks, temporary paths, size, and append.

## Decisions

- Add boolean path checks:
  - `isFile "path" -> $ok`
  - `isDir "path" -> $ok`
- Add temporary path helpers:
  - `tempFile -> $path`
  - `tempDir -> $path`
- Add file size helper:
  - `size "file" -> $bytes`
- Add file append helper:
  - `append "file" $text`
- Keep existing `write "file" $text` behavior; it already exists and overwrites content.
- `append "file" $text` creates parent directories and appends text without deleting existing content.
- File append appends the evaluated text plus one trailing newline.
- Preserve existing list `append` behavior: `append $list, "value"` remains list mutation.
- Single-path file helpers also accept scalar path variables, so `tempFile -> $path` can be used with `write $path`, `append $path`, `read $path -> $text`, `size $path -> $bytes`, `isFile $path -> $ok`, and `remove $path`.
- Resolve relative file paths from the current Shelly script directory.

## Tasks

- [x] Implement helper commands and checker validation.
- [x] Add CLI tests.
- [x] Document helpers in file docs and command docs.
- [x] Verify formatting and tests.

## Evidence

- Added runtime commands:
  - `isFile "path" -> $ok`
  - `isDir "path" -> $ok`
  - `tempFile -> $path`
  - `tempDir -> $path`
  - `size "file" -> $bytes`
  - `append "file" $text`
- Preserved `write "file" $text` and documented it alongside `append`.
- Added variable path support for single-path file helpers, including temp paths.
- Preserved list append/remove dispatch:
  - `append $list, "value"`
  - `remove $list[first]`
- Added checker validation and tests for helper syntax.
- Verification passed:
  - `cargo fmt --check`
  - `cargo test file_helper_commands_check_file_dir_temp_size_and_append --test cli_smoke`
  - `cargo test size_fails_for_directory_and_append_preserves_list_append --test cli_smoke`
  - `cargo test` with 113 tests passed
  - manual smoke with `tempFile`, `write`, `append`, `read`, `size`, `isFile`, and `remove` passed
  - README relative markdown/docs links passed
- Native Gentle review inspect remained blocked by `package-local-binary-missing`.
