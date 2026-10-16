# Shelly LSP diagnostics

Feature: diagnostics-first reusable stdio language server.

## Decisions

- First milestone supports diagnostics only, not completion, hover, or navigation.
- The server uses JSON-RPC/LSP over stdio and never executes Shelly scripts.
- It must validate unsaved document contents and preserve CLI check behavior.
- Workspace module diagnostics and symbol features are deferred.

## Tasks

- [x] Extract reusable checker and diagnostics APIs without changing CLI behavior.
- [x] Add LSP stdio lifecycle and diagnostics publishing tests.
- [x] Implement the diagnostics-only language server.
- [x] Document invocation and editor setup.
- [x] Verify CLI and LSP suites.

## Evidence

- LSP smoke test: 1 passed.
- CLI smoke tests: 201 passed.
- Full test suite: 202 passed.
- `cargo fmt --check`, `cargo check`, and `git diff --check`: passed.
- Separate CLI/LSP entrypoints produce no Cargo duplicate-target warning.
- No commit created; awaiting explicit user authorization.
