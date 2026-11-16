# Shelly tables

Basic terminal tables with styled expression cells and matching editor support.
Feature branch: `feature/tables`.

## Specs

S1. "tablas basicas por el momento . vamos a ver sintaxis"
S2. "me gustaria poder usar style dentro de tablas , por sinquiero texto de color , lo mismo si quiero usar variables o constantes como valores tambien"
S3. "perfecto . hazlo , y recuerda actualizar shelly-nvim y shelly-vscode para esto nuevo"
S4. "debes hacer las releases recuerda, y generar el vsix , todo en main . y dime , como quedó el tema de la sintaxis y que es lo que se puede hacer con table"

## Tasks

T1 | S1 | Shelly parser, block validation, formatter, and focused RED tests | verified: focused formatter + CLI tests; commit `097a16b`
T2 | S1-S2 | Table model and terminal renderer with border modes and expression cells | verified: focused tests + full `cargo test`; commit `097a16b`
T3 | S1-S2 | Public behavior tests and language documentation | verified: docs links + focused behavior tests; commit `da2f751`
T4 | S3 | Neovim syntax/indentation and tests | verified: headless structured-syntax test; commit `4808768` in `shelly-nvim`
T5 | S3 | VS Code grammar and tests | verified: focused extension tests + validate + build; commit `84fb614` in `shelly-vscode`
T6 | S4 | Prepare Shelly v3.15.0 on main (merge and version commit) | prepared on main: commit `d339133`
T7 | S4 | Prepare Neovim v0.7.0 on main (merge and version/tag readiness) | prepared on main: commit `b68e149`
T8 | S4 | Prepare VS Code v0.7.0 on main and generate the VSIX | prepared on main: commit `75331d2`; VSIX pending one-shot validation
T9 | S4 | Run the one-shot release validation, publish exact tags/releases, and independently verify artifacts | commit pending

## Log

L1. perfecto . hazlo , y recuerda actualizar shelly-nvim y shelly-vscode para esto nuevo
L2. Authorized external editor access: `/home/soyagvs/proyectos/personal/shelly-nvim` and `/home/soyagvs/proyectos/personal/shelly-vscode`.
L3. Agreed design: basic `table ... end` data tables; `column`, `row`, and `footer`; border modes `ascii`, `unicode`, and `rounded`; ordinary Shelly expressions, including variables, constants, and `style(STYLE, VALUE)`, in cells. The renderer aligns on visible text width and preserves the existing color policy.
L4. T2 reopened after public verification found a stale ASCII-output expectation: the implementation correctly renders a separator under the header, while one earlier test did not expect it.
L5. Slice 1: `097a16b feat(tables): add terminal table blocks` (442 authored lines; cohesive behavior and its tests).
L6. Slice 2: `da2f751 docs(tables): document terminal tables` (86 authored lines, including the tracked feature document).
L7. Editor work units: `4808768 feat(syntax): highlight table blocks` in `shelly-nvim`; `84fb614 feat(grammar): support table blocks` in `shelly-vscode`.
L8. debes hacer las releases recuerda, y generar el vsix , todo en main . y dime , como quedó el tema de la sintaxis y que es lo que se puede hacer con table
L9. Release decision: minor line selected — Shelly v3.15.0, VS Code v0.7.0, and matching Neovim v0.7.0 release/tag.
L10. Core release preflight initially invoked `cargo check --locked` after the version bump; it correctly refused the required Cargo.lock refresh, so zero validation tests ran. `cargo check --offline` repaired Cargo.lock.
L11. User authorized one fresh, bounded release-validation attempt after the Cargo.lock repair. Preparation will complete in all repositories before that single validation/publish task.
L12. Shelly feature was merged to `main`; v3.15.0 preparation commit: `d339133 chore: prepare v3.15.0`.
L13. Neovim feature was merged to `main`; v0.7.0 changelog preparation commit: `b68e149 chore: prepare v0.7.0`.
L14. VS Code feature was merged to `main`; v0.7.0 package, lockfile, README, and VSIX verifier preparation commit: `75331d2 chore(release): prepare VS Code extension 0.7.0`.
L15. The one-shot coordinated validation started: all Shelly checks completed successfully, then the Neovim phase stopped before probes because `tests/headless-custom-command.sh` is not executable (`Permission denied`, exit 126). VS Code validation, VSIX generation, tags, pushes, and publication did not run.
L16. User authorized one fresh coordinated validation attempt using `bash tests/headless-custom-command.sh`; publication remains conditional on all gates passing.
L17. The second coordinated validation again completed all Shelly checks successfully, then stopped before remaining probes because `tests/headless-icons.sh` is also not executable (`Permission denied`, exit 126). VS Code validation, VSIX generation, tags, pushes, and publication did not run.
L18. User authorized one final coordinated validation attempt using `bash` for both non-executable Neovim scripts; no retry follows if this attempt fails.
