# Unix PATH setup

Feature: make the Unix Shelly installer configure the user PATH automatically when needed.

## Approved direction

When the selected install directory is absent from `PATH`, `install.sh` detects the user's supported shell profile and appends an idempotent `PATH` export. The installer must explain the change and tell the user to open a new terminal. Existing PATH entries must never be duplicated.

## Tasks

- [x] Map safe shell/profile detection and installer edge cases.
- [x] Add isolated installer coverage for PATH profile setup.
- [x] Implement idempotent PATH profile setup in `install.sh`.
- [x] Document automatic PATH setup and fallback behavior in README.
- [x] Validate shell syntax and installer behavior without modifying the developer environment.

## Constraints

- Do not modify a shell profile when the install directory is already in PATH.
- Do not overwrite existing profile contents.
- Do not commit unless explicitly requested.

## Evidence

- RED observed: the isolated Bash profile setup assertion failed before implementation.
- `bash tests/install_sh_test.sh` passed with temporary HOME/profile isolation.
- `sh -n install.sh` passed.
- `git diff --check` passed.
- Coverage includes Bash/Zsh idempotent profile blocks, existing-PATH no-op, and Fish/unknown-shell fallback without profile mutation.
