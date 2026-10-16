# Fix versioned installer assets

Feature: make Unix and Windows installers resolve the latest GitHub release tag and download the versioned release archive that contains Shelly and the LSP.

## Decisions

- Use the GitHub Releases API to read the latest `tag_name` when no explicit `SHELLY_VERSION` is set.
- Build the archive name from platform target and tag: `shelly-<target>-<tag>.<extension>`.
- Keep explicit version installs deterministic and use the same versioned naming convention.
- Extend installer regression coverage before changing implementation.

## Tasks

- [x] Add failing regression coverage for versioned release asset resolution.
- [x] Update Unix and Windows installers to resolve versioned assets.
- [x] Run focused and full verification.
- [x] Commit the installer fix.

## Evidence

- RED observed: `cargo test --test installers` failed as expected because `install.sh` lacks versioned archive construction; 1 test passed and 1 failed (exit 101).
- GREEN observed: both installers resolve latest release tags through the GitHub Releases API, construct versioned archive names, and `cargo test --test installers` passed (2 tests).
- Independent verification passed: focused installer tests, full `cargo test`, and `git diff --check`.
- Installer fix committed as `3fc91e61c09d14ddfb3f9a978aa82e38fee12f1d` (`fix(installer): resolve versioned release assets`).
