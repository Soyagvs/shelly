# Project structure

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Initialize a project

A single `.shelly` file is enough when you have one small script and do not need project metadata, tests, or a standard layout. Run it directly:

```sh
shelly hello.shelly
```

Use `shelly init` when you want a project starter layout, a `Shelly.toml` manifest, or the built-in test discovery. It creates a convenient starting point, but its folders are optional extension points—not a required architecture. You can begin with one file and add only the folders that make your project easier to understand:

```sh
mkdir my-shelly-project
cd my-shelly-project
shelly init
shelly main.shelly
```

`shelly init` uses the current directory and accepts no project-name argument. It preflights every scaffold target, so if any target already exists it fails without changing files.

A fresh scaffold contains:

```text
Shelly.toml
main.shelly
README.md
.gitignore
tests/
consts/
functions/
utils/
setup/
scripts/
modules/
domains/
```

- `Shelly.toml` — the manifest: it names the project, identifies its entry script, and can configure test discovery. It is required by the manifest-based forms of `shelly run`, `shelly check`, and `shelly test`.
- `main.shelly` — the user-facing entry point.
- `README.md` — starter project documentation.
- `.gitignore` — ignores generated files, including `logs/`.
- `tests/` — scripts that verify behavior. Add recursively discovered `*_test.shelly` files here or in nested directories; run them with `shelly test`.
- `consts/` — named values shared by several scripts.
- `functions/` — reusable actions that scripts can call.
- `utils/` — small helpers used in different parts of the project.
- `setup/` — code that prepares the environment when a script starts.
- `scripts/` — extra command entry points people can run directly.
- `modules/` — reusable building blocks that do not belong to one business area.
- `domains/` — code organized around a larger area of work, such as releases or documentation.

Each scaffold directory contains a `.gitkeep` marker so the empty directory can be tracked. `logs/` is ignored and generated lazily beside the running script when logging needs it; `init` does not create it.

A natural growth path is: start with one script; extract repeated or reusable code into imported files; then use purpose-oriented folders and domain groupings when the project has several commands or areas of work. None of those later stages is required.

Import and include paths are resolved relative to the file that contains the statement. For example, a file in `scripts/` uses `../consts/app.shelly` because `..` moves from `scripts/` back to the project root, where `consts/` is a sibling folder.

## Which command should I use?

- **Run one file directly:** `shelly path/to/script.shelly`. This is the simplest choice and does not need `Shelly.toml`.
- **Run the project entry point:** `shelly run`. This reads `Shelly.toml` in the current directory and uses `[project].entry`.
- **Check without executing:** `shelly check path/to/script.shelly` checks an explicitly named file and does not need a manifest. `shelly check` with no path reads `Shelly.toml` in the current directory.
- **Run project tests:** `shelly test`. This uses the test settings in `Shelly.toml` in the current directory, so a manifest is required.

## Shelly.toml

The required `[project]` fields are `name`, `version`, and `entry`. The optional `[test]` table controls test discovery: `directory` defaults to `tests`, and `pattern` defaults to `*_test.shelly`.

```toml
[project]
name = "my-shelly-project"
version = "0.1.0"
entry = "main.shelly"

[test]
directory = "tests"
pattern = "*_test.shelly"
```

Run the configured entry point from the project directory with `shelly run`. It reads `Shelly.toml` from the current directory and resolves `[project].entry` relative to that directory:

```sh
shelly run
```

Validate the configured entry without running it by omitting the check path. `shelly check` reads `Shelly.toml`, resolves `[project].entry`, and checks the entry script without executing its commands:

```sh
shelly check
```

For a script outside the manifest, provide an explicit path. JSON diagnostics are also available:

```sh
shelly check path/to/script.shelly
shelly check --format json path/to/script.shelly
```

`check` validates the selected script and its full reachable `import`/`include` graph without executing commands. It reports missing dependencies, errors found in transitive dependencies, and import/include cycles. The check uses existing module syntax; it does not introduce new syntax. With `--format json`, output is a JSON array of objects containing `file`, `line`, `column`, `severity`, and `message`; success produces `[]`, while any diagnostic returns a failing status.

Forward arguments to the entry script only after the `--` separator:

```sh
shelly run -- --dry-run input.txt
```

The separator is mandatory before forwarded arguments; `shelly run --dry-run` is invalid. For an explicit script path, run it directly instead:

```sh
shelly --dry-run main.shelly
```

## Tests

Create a self-contained test script at `tests/name_test.shelly` after initialization. It defines the values it uses, but it still runs inside the Shelly project and its `Shelly.toml` when invoked by `shelly test`:

```shelly
$value = "ready"
$items = ["one", "two"]
assertEqual $value "ready"
assertTrue true
assertFalse false
assertContains $items "two"
```

Tests are ordinary Shelly scripts. `shelly test` recursively discovers `*_test.shelly` files, executes each in isolation, and prints a PASS/FAIL summary. The command exits non-zero if any test fails or cannot run. Assertions stop execution on failure and are intended for tests; use `if` or `try`/`catch` for normal or recoverable flow.

`assertEqual` compares values, `assertTrue` requires a truthy value, `assertFalse` requires a falsey value, and `assertContains` checks that a collection contains a value.

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

In plain language, `import` brings reusable exported values or functions into a script without treating the imported file as startup code. Use `include` when you deliberately want the target file to run as part of the current script and share its scope. See [Modules](modules.md) for the detailed behavior and API rules.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
