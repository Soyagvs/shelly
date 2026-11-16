# Composable tables and measurement

Nested table control flow, monotonic timing, editor integration, and a coordinated release.
Feature branch: `feature/composable-tables-measure`.

## Specs

S1. "todo eso. que funcione naturalmente en cualquier funcion o bloque y que bloques funcionen dentro de tables. y tambien lo de measure. recuerda documentar todo. actualizar shelly-nvim y shelly-vscode y el nuevo vsix, todo en main pusheado con nueva release"
S2. "`row` dentro del bloque"

## Tasks

T1 | S1 | General block collection, validation, and formatter support for composable `table` and `measure` blocks | verified: focused CLI + formatter tests; commit pending
T2 | S1-S2 | Table row accumulation through nested control flow; fix column alignment for body rows | verified: focused CLI tests; commit pending
T3 | S1 | Monotonic `measure -> $milliseconds ... end` runtime and deterministic tests | verified: 5 focused CLI tests; commit pending
T4 | S1 | LSP/CLI coverage and complete language documentation | verified: docs diff/link checks; commit pending
T5 | S1 | Neovim syntax, indentation, and tests | verified: headless structured-syntax test; commit pending
T6 | S1 | VS Code grammar, indentation, tests, and VSIX packaging preparation | verified: focused tests + validate + build; commit pending
T7 | S1 | Merge to main, prepare minor releases, validate and publish | commit pending
T8 | S1 | Independently verify published release artifacts | commit pending

## Log

L1. todo eso. que funcione naturalmente en cualquier funcion o bloque y que bloques funcionen dentro de tables. y tambien lo de measure. recuerda documentar todo. actualizar shelly-nvim y shelly-vscode y el nuevo vsix, todo en main pusheado con nueva release
L2. Selected dynamic-row model: `row` inside a table control-flow block appends a row to the active table.
L3. Existing v3.15.0 limitation discovered before this feature: declared column alignment applies to headers/footer but not body rows; T2 corrects it.
