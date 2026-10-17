# Contributing to Shelly

Thanks for helping improve Shelly, a Rust implementation of a readable scripting language. This guide is the shortest reliable path from a local change to a reviewable pull request.

## Quick path

1. Install a stable [Rust toolchain](https://www.rust-lang.org/tools/install).
2. Clone the repository and work from its root.
3. Read the relevant language or project documentation before changing behavior: [language overview](docs/language.md), [project structure](docs/project-structure.md), and [documentation index](docs/index.md).
4. Make a focused change, adding or updating the nearest test when behavior changes.
5. Run the validation in [Verified validation](#verified-validation).
6. Open a focused pull request using the checklist below.

> **Important:** Releases are maintainer-owned. Contributors must not create tags, publish artifacts, or perform releases.

## Repository map and scope

| Path | What belongs there |
| --- | --- |
| [`src/`](src/) | Shelly interpreter, parser, runtime, values, CLI, updater, and LSP implementation |
| [`tests/`](tests/) | Rust integration tests for CLI behavior, language features, LSP, installers, packaging, and updates |
| [`docs/`](docs/) | User-facing language and project documentation |
| [`examples/`](examples/) | Small, runnable Shelly scripts |
| [`scripts/`](scripts/) | Maintainer and packaging helpers; do not broaden release scope in a feature PR |
| [`packaging/`](packaging/) | Packaging metadata, including the Mason package definition |
| [`Cargo.toml`](Cargo.toml) | Package metadata, Rust edition, license, and dependencies |
| [`LICENSE`](LICENSE) | GPL-3.0 license text |

Keep changes narrow. Do not edit generated or unrelated files, workflows, runtime behavior, tests, or task tracking unless the contribution specifically requires that scope and the change is clearly explained in the PR.

## Prerequisites and setup

You need a stable Rust toolchain with Cargo and a working checkout of this repository. From the repository root, inspect the existing [README](README.md) for user-facing conventions and choose the smallest relevant area of the codebase from the map above.

For language changes, update the appropriate page under [`docs/`](docs/) and, where useful, add a minimal example under [`examples/`](examples/). For project behavior, consult [`docs/project-structure.md`](docs/project-structure.md) before changing paths, manifests, or discovery rules.

## Verified validation

Run these commands from the repository root:

```bash
cargo fmt --check
cargo test
cargo run -- examples/conditionals.shelly
```

The sample script should print `enabled branch` and `interpolation works`.

### Targeted tests

Use a focused integration test while iterating, then run the full suite before requesting review:

```bash
cargo test --test cli_smoke
cargo test --test lsp_features
cargo test --test lsp_smoke
cargo test --test lsp_symbols
cargo test --test lsp_syntax
cargo test --test release_packaging
```

Choose the relevant target rather than running unrelated checks repeatedly. Changes to updater behavior should also exercise the [`tests/update.rs`](tests/update.rs) integration target; installer changes should exercise [`tests/installers.rs`](tests/installers.rs). These use the same Cargo test form, for example `cargo test --test update`.

## Documentation and compatibility

- Keep documentation examples executable and consistent with the current Shelly syntax.
- Link to an existing page instead of duplicating its contract.
- Explain user-visible behavior, errors, defaults, and compatibility implications.
- Preserve existing CLI, script, manifest, and LSP behavior unless the change intentionally updates that contract.
- When behavior changes, update the relevant documentation and tests together.
- Keep examples and docs free of machine-specific paths, credentials, and network assumptions.

## Issue and pull request checklist

Before opening a PR:

- [ ] The change has a focused purpose and does not include unrelated cleanup.
- [ ] Relevant source, documentation, examples, and tests are updated together.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo test` passes.
- [ ] The sample script and any relevant targeted tests pass.
- [ ] The PR describes compatibility or migration impact, or says that none is expected.
- [ ] The PR lists validation commands and notable limitations.
- [ ] No secrets, generated artifacts, tags, releases, or published packages are included.

## Release boundaries

Maintainers own version decisions, release notes, tags, binaries, checksums, and publication. Contributors should submit changes for review and may update release-related documentation when requested, but must not tag, publish, upload artifacts, or run a release. See the release and packaging references in [`README.md`](README.md), [`scripts/release-verify.sh`](scripts/release-verify.sh), and [`packaging/mason-registry/packages/shelly-lsp/package.yaml`](packaging/mason-registry/packages/shelly-lsp/package.yaml).

## License

Shelly is distributed under the [GNU General Public License v3.0 or later (GPL-3.0-or-later)](LICENSE). Contributions must be compatible with that license.
