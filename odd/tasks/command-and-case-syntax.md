# Command and case syntax consistency

Feature: normalize command argument syntax and case alternatives around existing language semantics.

## Decisions

- Commands use whitespace-separated arguments; commas do not delimit command arguments.
- Commas remain valid inside data structures, function signatures, selectors, and multi-capture destinations.
- Multiple `case` alternatives use existing boolean `or`: `when VALUE or VALUE`.
- Comma-separated `when VALUE, VALUE` alternatives are unsupported.

## Tasks

- [x] Map `case` parsing, checking, tests, and documentation references.
  - Evidence: both runtime block collection and checker validation currently split `when` alternatives by commas in `src/lib.rs`; command atoms are already whitespace-delimited.
- [x] Implement `when ... or ...` alternatives with RED/GREEN coverage and reject comma alternatives.
  - Evidence: focused RED exposed comma-based case parsing; `when VALUE or VALUE` passed after implementation and comma alternatives fail at runtime and under `check`.
- [x] Correct command/case documentation and verify the full suite.
  - Writer verification: focused case syntax test and full `cargo test` (237 tests) passed; independent verification confirmed 214 CLI smoke tests and 254 total tests passing.
- [x] Format the implementation and reconcile historical task documents that preserve obsolete comma examples.
  - `cargo fmt --check` passed after formatting; historical task documents now use current command and case syntax, including the `ask` task's capture form.
- [x] Complete documentation audit.
  - README, canonical docs, and all ODD task documents contain no stale comma-separated command examples or comma-separated `when` alternatives. Explanations of rejected syntax and structural commas remain intentionally documented.

## Evidence

- Documentation audit found command comma syntax in `docs/files.md` and an overly broad comma grammar statement in `docs/language.md`; `docs/conditionals.md` uses comma-separated `when` alternatives.
