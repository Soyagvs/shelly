# Project structure

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Start with one script. Add folders only when names and reuse make the project easier to read.

```text
hello.shelly
```

```shelly
shellout "Hello from Shelly"
```

## Rules

- Keep the entry point obvious: use `main.shelly` for project-style scripts.
- Use `import` for reusable files that `export` constants or functions.
- Use `include` only for files that are meant to run in place, such as setup.
- Put stable values in `consts/` and reusable functions in `functions/` or `utils/`.
- Put command entry points in `scripts/`; keep shared code outside them.
- Add `modules/` or domain folders only when generic folders stop being clear.

## Tiny scripts

Use one file when the script has one job and no shared helpers.

```text
rename-downloads.shelly
```

```shelly
$TARGET = "downloads"

if exists $TARGET
    shellout "ready: $TARGET"
end
```

## Small modular projects

Use `main.shelly` plus a few imported modules when helpers become reusable.

```text
project/
  main.shelly
  consts/
    paths.shelly
  functions/
    report.shelly
```

```shelly
# main.shelly
import "consts/paths.shelly"
import "functions/report.shelly"

call printReport($REPORT_DIR)
```

```shelly
# consts/paths.shelly
export $REPORT_DIR = "reports"
```

```shelly
# functions/report.shelly
export fn printReport($path) {
    shellout "report: $path"
}
```

## Medium projects

Separate startup setup from reusable modules. `setup/` files are good `include` targets because they intentionally execute.

```text
project/
  main.shelly
  consts/
    app.shelly
  functions/
    build.shelly
  utils/
    text.shelly
  setup/
    environment.shelly
  scripts/
    clean.shelly
    release.shelly
```

```shelly
# main.shelly
import "consts/app.shelly"
import "functions/build.shelly"
include "setup/environment.shelly"

call build($APP_NAME)
```

```shelly
# setup/environment.shelly
$mode = "local"
shellout "mode: $mode"
```

```shelly
# scripts/clean.shelly
import "../consts/app.shelly"

shellout "cleaning $APP_NAME"
```

## Advanced projects

For large automation, group by domain. Keep each domain's public API in exported files, and keep entry points thin.

```text
project/
  main.shelly
  consts/
    app.shelly
  setup/
    environment.shelly
  modules/
    git.shelly
    markdown.shelly
  domains/
    release/
      plan.shelly
      publish.shelly
    docs/
      index.shelly
  scripts/
    release.shelly
    docs-check.shelly
```

```shelly
# scripts/release.shelly
import "../domains/release/plan.shelly"
import "../domains/release/publish.shelly"
include "../setup/environment.shelly"

call planRelease($mode)
call publishRelease()
```

```shelly
# domains/release/plan.shelly
export fn planRelease($mode) {
    shellout "planning release for $mode"
}
```

## Import versus include

Use structure to make intent visible.

| Need | Use | Folder fit |
| --- | --- | --- |
| Share exported constants | `import` | `consts/` |
| Share exported functions | `import` | `functions/`, `utils/`, `modules/`, domain folders |
| Run setup code in the caller's flow | `include` | `setup/` |
| Start a user-facing command | run the file directly | `main.shelly`, `scripts/` |

```shelly
import "utils/text.shelly"
include "setup/environment.shelly"

call title("daily report") -> $heading
shellout "$heading in $mode"
```

`import` should be quiet: imported files expose exports and do not run private top-level code. `include` should be intentional: the target file runs in the current scope.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
