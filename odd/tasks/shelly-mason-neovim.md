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
- [x] Verify release artifacts and publish the Shelly release.
- [ ] Submit the official Mason registry contribution (blocked by upstream admission requirements).

## Evidence

- Focused distribution tests: 5 passed.
- `cargo fmt --check`, `cargo test`, `cargo check`, Mason verifier, and `git diff --check`: passed.
- `v3.3.1` release artifacts exist for Linux x86_64, macOS x86_64, and macOS aarch64.
- Mason's current contribution policy requires a new package to meet at least one condition: 100 GitHub stars, 5,000 VS Code Marketplace downloads, approval in `nvim-lspconfig`, or an official recommendation by a reputable organization. Shelly currently has 1 GitHub star and no documented qualifying condition.
- The current local package metadata also needs the upstream-required YAML document marker and a versioned purl `source.id` before it could be submitted.
