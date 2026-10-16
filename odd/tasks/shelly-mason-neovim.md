# Shelly Mason and Neovim distribution

Feature: zero-configuration Neovim LSP via Mason for Linux x86_64 and macOS x86_64/aarch64.

## Decisions

- Release `shelly-lsp` alongside Shelly for Linux x86_64, macOS x86_64, and macOS aarch64.
- Neovim integration auto-attaches Shelly buffers without user Lua setup.
- Mason installation must use published release assets and a registry package.
- The user authorized publication after local verification.

## Tasks

- [x] Add release artifacts and checksums for shelly-lsp on the three Unix targets.
- [x] Add auto-configuring Neovim plugin and tests/documentation.
- [x] Create and validate Mason package registry metadata.
- [ ] Verify release artifacts and publish release/registry contribution.

## Evidence

- Focused distribution tests: 5 passed.
- `cargo fmt --check`, `cargo test`, `cargo check`, Mason verifier, and `git diff --check`: passed.
- GitHub release/tag and Mason upstream submission remain external publication steps.
