# Nested collection mutation

Feature: add predictable mutation of nested lists and dictionaries.

## Decisions

- Collections use value semantics: assigning a collection copies it; there are no implicit shared mutable references.
- List indexes are one-based; zero and out-of-range indexes are errors.
- `set`, `append`, and `delete` are the v1 mutation operations.
- Mutation command arguments are whitespace-separated: `set PATH VALUE` and `append PATH VALUE`; commas are unsupported so commands share one syntax.
- Dictionary terminal keys may be created or replaced; every intermediate path segment must already exist and have the required container type.
- Lists never auto-expand; use `append` to add a final element.
- `delete` is strict and deleting a list entry compacts following indexes.
- Support dot paths for simple keys and bracket-string paths for arbitrary dictionary keys.
- Defer insert, prepend, slicing, and range replacement.

## Tasks

- [x] Map existing collection reads, assignments, parser/checker behavior, and test conventions.
  - Evidence: mixed nested reads and dictionary paths already exist in `src/lib.rs`; top-level list mutation is one-based and strict. The only unresolved language decision is the concrete command argument grammar.
- [x] Add nested-path mutation with focused RED/GREEN coverage.
  - Evidence: RED reported `unknown command or invalid syntax: set ...`; focused nested-mutation tests passed 3/3 after implementation, and existing top-level list mutation tests continued to pass.
- [x] Document v1 mutation semantics and deferred operations.
  - Updated `docs/commands.md`, `docs/lists.md`, and `docs/dictionaries.md`.
- [x] Run focused and full verification.
  - Final independent verification passed `cargo test --test cli_smoke` (212 tests), full `cargo test` (252 tests), `cargo fmt --check`, and `git diff --check`.
- [x] Normalize collection mutation arguments to whitespace and reject legacy commas.
  - Evidence: focused mutation tests observed RED under the prior comma-only parser, then passed with whitespace commands. Documentation now uses whitespace consistently for `set`, `append`, and `delete`.
- [x] Normalize legacy top-level `prepend` syntax and reconcile its documentation.
  - Evidence: focused prepend tests observed RED under comma syntax, then passed with `prepend $list VALUE`; docs now distinguish top-level prepend from deferred nested prepend.
- [x] Verify final syntax normalization.
  - Independent verification passed `cargo test --test cli_smoke` (213 tests), full `cargo test` (247 tests), and `git diff --check`; the follow-up `cargo fmt --check` passed after formatting only `src/lib.rs` and `tests/cli_smoke.rs`.

## Evidence

- Product design agreed before implementation.
- RED: the first focused test failed with `unknown command or invalid syntax: set ...`.
- GREEN: nested-mutation tests passed after implementation; existing list mutation behavior remained covered.
- Syntax normalization initially exposed file-append dispatch regressions; quoted file paths, variable file paths, bare variable collections, and nested collection paths now route correctly, while comma mutation syntax is rejected.
