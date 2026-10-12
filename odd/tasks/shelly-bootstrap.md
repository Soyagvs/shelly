# Shelly bootstrap

Feature: create the initial Shelly project scaffold and MVP language skeleton.

## Decisions

- Product name: Shelly.
- Script extension: `.shelly`.
- Implementation language: Rust.
- Initial syntax has no semicolons.
- Variables use `$name = value`.
- `shellout` writes output.
- `shellin $var` reads input into a variable.
- `command "command"` executes an external command.
- Historical note: the initial MVP planned `capture`, but it was later removed. Current syntax is `command "..." -> $var`; see `odd/tasks/shelly-command-capture-arrow.md`.

## Tasks

- [x] Create Rust CLI project scaffold under `/home/soyagvs/proyectos/personal/shelly`.
- [x] Implement MVP interpreter for `.shelly` files with variables, interpolation, `shellout`, `shellin`, `command`, and the original capture command later superseded by `command "..." -> $var`.
- [x] Add automated tests covering the initial Shelly syntax.
- [x] Add README with syntax examples and MVP scope.

## Evidence

- Scaffold: `cargo test` passed via worker; 1 smoke test passed. Files: `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `tests/cli_smoke.rs`, `README.md`, `.gitignore`.
- Interpreter/tests/docs: worker implemented line-oriented `.shelly` runtime and expanded tests; `cargo test` passed with 7 integration tests.
- Manual verifier run: temporary `.shelly` script executed successfully through `cargo run -- <script>`.
- Final verification: `cargo fmt --check` passed; `cargo test` passed with 7 integration tests.
- Git status note: `/home/soyagvs/proyectos/personal/shelly` is currently untracked under the enclosing `/home/soyagvs` git root.
- Language usage documentation added in `docs/language.md`; `README.md` now links to it from the MVP syntax section.
- Identifier decision documented: variables use `$name` and are case-insensitive; constants use `$$NAME`, must be uppercase, and identifiers must start with a letter before allowing letters, numbers, or `_`.
