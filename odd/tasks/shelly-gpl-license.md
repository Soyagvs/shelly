# GPL licensing

Feature: relicense Shelly under GNU GPL version 3 or later.

## Approved direction

Replace the MIT license with the unmodified GNU GPL v3 text. Declare `GPL-3.0-or-later` in Cargo package metadata. The copyright owner confirmed authority over the current codebase.

## Tasks

- [x] Replace the repository license with the official GPL v3 text.
- [x] Declare the SPDX license identifier in Cargo metadata.
- [x] Verify license text integrity and package metadata.
- [x] Document GPL-3.0-or-later in README.

## Constraints

- Use the official, unmodified GNU GPL v3 license text.
- Do not commit unless explicitly requested.

## Evidence

- Work-unit commit: `44b4d0e chore: relicense Shelly under GPL`.
- LICENSE matches the official GNU GPL v3 text (Version 3, 29 June 2007).
- Cargo metadata declares `GPL-3.0-or-later`.
- `git diff --check` passed.
