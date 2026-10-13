# Shelly exists and contains predicates

Feature: add readable built-in predicates for file existence and containment checks.

## Decisions

- Keep `include` only for loading/executing Shelly files.
- Add `exists "path"` for file/directory existence checks.
- Add `contains $value "thing"` for list and string containment checks.
- Support predicates directly in `if`/`elif` conditions and with `and` / `or` / `not`.
- Support command-style result capture with `-> $var` for scripts that want to store the boolean.

## Tasks

- [x] Implement `exists` predicate and capture form.
- [x] Implement `contains` predicate and capture form.
- [x] Add tests for direct conditions, `not`, logical composition, capture, and invalid usage.
- [x] Update docs and examples where appropriate.
- [x] Run verification.

## Evidence

- Added runtime support for `exists "path"` and `contains $value "thing"` in conditions.
- Added capture forms:
  - `exists "path" -> $hasPath`
  - `contains $value "thing" -> $hasThing`
- Added checker validation for predicate conditions and capture forms.
- Added tests for existence, missing paths, `not exists`, string/list `contains`, logical composition, capture, invalid predicate forms, and paths containing `->`.
- Updated docs:
  - `README.md` docs table includes the predicates guide.
  - `docs/language.md` links predicates and calls out logical operator priority.
  - `docs/conditionals.md` expands `and` / `or` / `not` priority and predicate composition examples.
  - `docs/commands.md` documents predicate capture as boolean capture and links to predicates.
  - `docs/predicates.md` documents `exists`, `not exists`, string/list `contains`, capture forms, composition, `include` distinction, and current limitations.
- Verification passed:
  - `cargo fmt --check`
  - `cargo test exists`
  - `cargo test contains`
  - `cargo test` with 94 tests passed
  - Manual smoke with `exists`, `contains`, `not`, `and`, capture forms, and `arrow->file.txt` path passed.
