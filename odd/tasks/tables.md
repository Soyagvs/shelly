# Shelly tables

Basic terminal tables with styled expression cells and matching editor support.
Feature branch: `feature/tables`.

## Specs

S1. "tablas basicas por el momento . vamos a ver sintaxis"
S2. "me gustaria poder usar style dentro de tablas , por sinquiero texto de color , lo mismo si quiero usar variables o constantes como valores tambien"
S3. "perfecto . hazlo , y recuerda actualizar shelly-nvim y shelly-vscode para esto nuevo"

## Tasks

T1 | S1 | Shelly parser, block validation, formatter, and focused RED tests | verified: focused formatter + CLI tests; commit `097a16b`
T2 | S1-S2 | Table model and terminal renderer with border modes and expression cells | verified: focused tests + full `cargo test`; commit `097a16b`
T3 | S1-S2 | Public behavior tests and language documentation | verified: docs links + focused behavior tests; commit pending
T4 | S3 | Neovim syntax/indentation and tests | verified: headless structured-syntax test; commit pending
T5 | S3 | VS Code grammar and tests | verified: focused extension tests + validate + build; commit pending

## Log

L1. perfecto . hazlo , y recuerda actualizar shelly-nvim y shelly-vscode para esto nuevo
L2. Authorized external editor access: `/home/soyagvs/proyectos/personal/shelly-nvim` and `/home/soyagvs/proyectos/personal/shelly-vscode`.
L3. Agreed design: basic `table ... end` data tables; `column`, `row`, and `footer`; border modes `ascii`, `unicode`, and `rounded`; ordinary Shelly expressions, including variables, constants, and `style(STYLE, VALUE)`, in cells. The renderer aligns on visible text width and preserves the existing color policy.
L4. T2 reopened after public verification found a stale ASCII-output expectation: the implementation correctly renders a separator under the header, while one earlier test did not expect it.
L5. Slice 1: `097a16b feat(tables): add terminal table blocks` (442 authored lines; cohesive behavior and its tests).
