# Shelly check and VS Code diagnostics

Feature: add safe syntax checking for Shelly scripts and surface diagnostics in the VS Code extension.

## Decisions

- Add `shelly check <script.shelly>` before VS Code diagnostics.
- `check` must parse/validate without executing `shellout`, `command`, `ask`, `shellin`, imports, or includes.
- Add machine-readable diagnostics with `--format json` for editor integrations.
- Keep CLI text output human-readable and JSON output stable enough for the extension.
- VS Code extension diagnostics should call the installed `shelly` binary and parse JSON output.

## Tasks

- [x] Add CLI parsing for `shelly check`.
- [x] Add safe checker and CLI tests in Shelly.
- [x] Add JSON diagnostic output for VS Code.
- [x] Add VS Code diagnostics integration.
- [x] Verify Shelly tests and extension package.
- [x] Publish/install updated extension package.

## Evidence

- Shelly runtime commit: `d7e66cf feat: add safe script checking`.
- Shelly tag pushed: `v3.0.3`.
- Shelly VS Code extension commit: `419438a feat: add Shelly diagnostics`.
- Shelly VS Code release published: `https://github.com/Soyagvs/shelly-vscode/releases/tag/v0.0.2`.
- VSIX asset published: `shelly-language-support-0.0.2.vsix`.
- Shelly worker added `shelly check <file>` and `shelly check --format json <file>` in `src/main.rs`.
- Shelly worker added CLI smoke tests for valid check, non-execution, missing `end`, `break` outside loop, `return` outside function, `call` without name, and JSON diagnostics.
- Shelly worker validation passed:
  - `cargo test --test cli_smoke check_ -- --nocapture`
  - `cargo fmt --check && cargo test`
- Parent/runtime verification passed:
  - `cargo fmt --check`
  - `cargo test` with 91 tests passed
  - manual missing-`end` JSON smoke returned `"message":"missing end for if"`
  - manual `command`/`shellout` check-mode smoke produced no script output
- VS Code worker added diagnostics integration in `src/extension.js`, version `0.0.2`, packaging metadata, and README diagnostics notes.
- VS Code worker validation passed:
  - `npm run validate && npm run build && npx --no-install @vscode/vsce package --no-dependencies`
- Parent/extension verification passed:
  - `npm run validate`
  - `npm run build`
  - `npm run package`
  - generated `shelly-language-support-0.0.2.vsix`
- GitHub CI for Shelly `main` commit `d7e66cf` passed.
- GitHub release-binaries workflow for `v3.0.3` was queued when checked.
