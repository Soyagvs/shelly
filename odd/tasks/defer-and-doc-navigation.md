# Add deferred cleanup and improve documentation navigation

Feature: add a portable `defer ... end` cleanup block and make Shelly's time/date documentation navigable as a book-like reference.

## Decisions

- `defer` registers cleanup for the enclosing execution scope and runs cleanup in LIFO order.
- Cleanup runs on successful scope exit, ordinary errors, and Shelly control signals (`exit`, `return`, `break`, `continue`); the original error or control signal remains authoritative if cleanup also fails.
- Documentation must make `now`, `timestamp`, `today`, and `formatDate` independently discoverable through a clear Time and date navigation path.

## Tasks

- [x] Implement checked and runtime `defer` semantics with focused CLI coverage.
- [x] Restructure time/date documentation navigation and add deferred-cleanup reference documentation.
- [x] Run focused and full verification.
- [x] Publish Shelly `v3.4.3`.

## Evidence

- Exploration mapped `run_lines`, block collection, and checker structures in `src/lib.rs` as the implementation surface.
- Existing docs contain date content but split entry points across helpers, date formatting, language guide, and README without a dedicated index path.
- Focused defer tests observed RED before implementation; focused and full CLI smoke tests passed after implementation (205 tests).
- Added `docs/index.md` as a categorized documentation table of contents, a standalone `docs/defer.md` reference, and bidirectional Time and date navigation across README and language references.
- Final verification passed: `cargo fmt --check`, all 236 tests, `git diff --check`, and 117 local Markdown targets/anchors.
- Published Shelly `v3.4.3` from `fe0853f`; all Linux/macOS/Windows release jobs passed and every archive plus SHA-256 asset is available.
