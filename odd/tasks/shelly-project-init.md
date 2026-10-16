# Shelly project initialization

Feature: add `shelly init` to create a full Shelly project scaffold in the current directory.

## Decisions

- `shelly init` creates the project in the current working directory; it does not accept or create a project-name subdirectory.
- The command creates `Shelly.toml`, `main.shelly`, `README.md`, `.gitignore`, and the directories `tests/`, `consts/`, `functions/`, `utils/`, `setup/`, `scripts/`, `modules/`, and `domains/`.
- Empty tracked directories contain `.gitkeep`.
- Generated `logs/` is ignored but not created.
- Initialization must fail safely rather than overwriting existing scaffold paths.

## Tasks

- [x] Map CLI and filesystem behavior; define failing integration tests for current-directory initialization and collision safety.
- [x] Implement `shelly init` scaffold creation.
- [x] Document project initialization and scaffold conventions.
- [x] Correct formatting in implementation and tests.
- [x] Correct stale legacy command syntax references.
- [x] Correct remaining legacy command examples in documentation.
- [x] Run focused and full verification; record evidence.

## Evidence

- Design: preflight every file, directory, and `.gitkeep` target before writing anything. Any existing target fails safely, leaving the directory untouched.
- RED: `cargo test --test cli_smoke init_` ran three tests; two scaffold tests failed as intended because `init` is currently treated as a script path.
- GREEN: `cargo test --test cli_smoke init_` passed all 3 focused initialization tests.
- Documentation: updated `README.md` and `docs/project-structure.md` with the quick path, scaffold conventions, collision safety, and lazy logs.
- Verification attempt: init tests passed (3/3), full suite passed (166/166), and diff whitespace checks passed. `cargo fmt --check` failed in `src/main.rs` and `tests/cli_smoke.rs`.
- Correction: formatted `src/main.rs` and `tests/cli_smoke.rs`; `cargo fmt --check` now passes.
- Repeat verification: format passed, init tests passed (3/3), and full suite passed (166/166). Verifier found two stale references to removed `command "..."` syntax in `docs/errors.md` and a capture-syntax diagnostic in `src/main.rs`.
- Correction: replaced legacy quoted-command examples in `docs/errors.md` and updated the capture diagnostic in `src/main.rs`; format passes.
- Final verification attempt: format passed, init tests passed (3/3), and full suite passed (166/166). Five stale legacy quoted-command examples remain in `docs/variables.md` and `docs/logging.md`; explanatory migration notes are intentional.
- Correction: migrated stale executable quoted-command examples in `docs/variables.md` and `docs/logging.md`; explanatory migration notes remain unchanged.
- Final verification passed:
  - `cargo fmt --check`
  - `cargo test --test cli_smoke init_` (3/3)
  - `cargo test` (166/166)
- No project-init or user-documentation findings. Three historical `odd/tasks/` records preserve legacy syntax as past-task evidence and were intentionally left unchanged.
- Native review was unavailable because the package-local Gentle AI binary is missing; assessment treated the candidate as unassessable, and the required independent verifier passed.
- No commit was created; committing requires explicit user authorization.
