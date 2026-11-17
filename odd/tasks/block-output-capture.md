# Universal block output capture

Tee normal block output into a variable without changing display behavior, then update editors and release.
Feature branch: `feature/block-output-capture`.

## Specs

S1. "bien . haz eso . luego actualiza shelly-nvim y shelly-vscide con releases y todo en main pusheado"
S2. "el -> $var , hace que no muestre su salida? eso esta mal . debe mostrar la salida , es solo guardar esa salida en una variable . para luego usarla"
S3. "bien , eso debe see asi universal . en cualquier cosa que haga , si lo muestro con shellout debe salir en terminal , ahora si hago algo y no me indico que lo muestre pero lo guatdo en variable , deberia funcionar de esa msnera , no mostrarlo y gustdarlo en variable"

## Tasks

T1 | S2-S3 | Add stdout tee capture for block/table destinations and CLI tests | verified: 3 focused CLI tests; commit pending
T2 | S1-S3 | Checker, formatter, LSP, and documentation | verified: formatter/LSP tests + docs links; commit pending
T3 | S1 | Neovim syntax/indent/tests | verified: headless structured-syntax test; commit pending
T4 | S1 | VS Code grammar/indent/tests and VSIX preparation | commit pending
T5 | S1 | Merge, release, and publish from main | commit pending
T6 | S1 | Independently verify release artifacts | commit pending

## Log

L1. bien . haz eso . luego actualiza shelly-nvim y shelly-vscide con releases y todo en main pusheado
L2. Universal rule: `-> $variable` stores output/result but never decides visibility. A visual producer such as `shellout` or `table` still writes to terminal; block destinations tee only its normally emitted stdout and assign after successful completion. Silent operations remain silent and retain their own result capture behavior.
L3. recuerda documentar bien todo esto
