# Coordinated Process Watch Release

## Objective
Repair Shelly CI and publish synchronized releases for the watch/wait feature: Shelly `v3.6.0`, shelly-nvim `v0.1.3`, and shelly-vscode `v0.1.2`, then create a reusable coordinated-release skill.

## Release decisions
- Shelly: `3.5.0` → `3.6.0` because watch/wait is a backward-compatible language feature.
- Neovim: `0.1.2` → `0.1.3` because syntax support is an incremental patch.
- VS Code: publish existing version `0.1.2`; its source and locally generated VSIX already use that version.

## Constraints
- The user explicitly authorized commits, tags, pushes, GitHub releases, and VSIX attachment.
- Every repository must be on `main` and pass its applicable release gate before tags/releases.
- Shelly CI must pass before releasing; formatting is currently known to fail.
- Do not publish a partial release set: prepare and validate all first, then tag/publish in dependency order.

## Tasks
- [x] CPR-1 — Repair Shelly formatting, bump to 3.6.0, and pass all release gates.
  - Outcome: updated Cargo metadata/lockfile and applied formatting-only corrections to watch/wait, LSP, and tests.
  - Evidence: `cargo fmt --check` passed; independent `cargo test` passed (253 tests); example run passed; `cargo test --locked --test update` passed (8 tests).
- [x] CPR-2 — Prepare Neovim 0.1.3 changelog and validate syntax support.
  - Outcome: added dated changelog notes for watch/wait syntax highlighting and support.
  - Evidence: `git diff --check` and headless Neovim syntax validation passed.
- [x] CPR-3 — Prepare VS Code 0.1.2 release documentation and artifact.
  - Outcome: added watch/wait release notes, corrected every versioned installation/download reference, and regenerated the VSIX.
  - Evidence: validation, build, 14 tests, package, VSIX verification, diff check, and independent release verification passed.
  - Artifact: `shelly-language-oficial-0.1.2.vsix` (849,730 bytes).
- [x] CPR-4 — Create the coordinated-release skill and refresh discovery.
  - Outcome: created a reusable gated release workflow skill and indexed it.
  - Evidence: registry refresh indexed 15 skills; frontmatter, headings, mandatory gates, and exact registry path passed structural validation.
- [x] CPR-5 — Commit/push release preparation, tag each exact release commit, publish GitHub releases, and verify remote assets.
  - Shelly: `e4e927bf03084e084f904b6c51754fa7eacd8b9f`, tag `v3.6.0`, CI and binary-release workflow passed; archives and checksums published. The Linux archive was independently verified to contain both `shelly` and `shelly-lsp`.
  - Neovim: `d37a98f8e6088c52a57ffa5a15ffbe1dab6f556b`, tag/release `v0.1.3` published.
  - VS Code: `770a742ebe4bafde5c1bee63d7878bea32c962bf`, tag/release `v0.1.2` published with `shelly-language-oficial-0.1.2.vsix` attached.

## Verification evidence
- Pending.

## Next step
Coordinated releases are published and verified. Future releases must load `skills/shelly-coordinated-release/SKILL.md` before any release action.
