# Shelly brace function syntax

Feature: replace old function syntax with parenthesized parameters and brace bodies.

## Decisions

- Remove old function definition syntax:
  - `fn name $arg`
  - body
  - `end`
- Add new function definition syntax:
  - `fn name($arg, $other = "default") {`
  - body
  - `}`
- Add exported function syntax:
  - `export fn name($arg) {`
  - body
  - `}`
- Remove old function call syntax:
  - `call name "arg"`
  - `call name "arg" -> $result`
- Add new call syntax:
  - `call name("arg")`
  - `call name("arg") -> $result`
- Keep explicit `call`; do not allow bare calls.
- Only function bodies use `{}` for now. Existing blocks (`if`, `for`, `while`, `try`, `log`, etc.) still use `end`.
- No backwards compatibility required because Shelly is still pre-adoption.

## Tasks

- [x] Implement function parser changes.
- [x] Implement call parser changes.
- [x] Update checker validation.
- [x] Update docs/examples/tests.
- [x] Verify formatting and tests.

## Evidence

- Added function definitions:
  - `fn name() { ... }`
  - `fn name($arg, $other = "default") { ... }`
  - `export fn name($arg) { ... }`
- Added function calls:
  - `call name()`
  - `call name("arg", 1, true)`
  - `call name("arg") -> $result`
- Removed old definition/call syntax with explicit errors.
- Updated docs and examples:
  - `docs/functions.md`
  - `docs/modules.md`
  - `example-01.shelly`
  - `examples/functions.shelly`
  - `examples/import-module.shelly`
  - `examples/module.shelly`
- Verification passed:
  - `cargo fmt --check`
  - `cargo test function` with 8 targeted tests passed
  - `cargo test` with 125 tests passed
  - README local docs-link check passed
  - manual smoke for `fn banner($name) { ... }`, `call banner("Ada")`, and return capture passed
- Native Gentle review inspect remained blocked by `package-local-binary-missing`.
