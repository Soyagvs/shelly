# Shelly language guide

[Documentation index](index.md) · [Deferred cleanup](defer.md) · [Time and date](index.md#time-and-date)

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Shelly scripts use the `.shelly` extension. One instruction per line. No semicolons.

````shelly
$name = "Ada"
$APP = "Shelly"

shellout "hello $name from $APP"

shellout ```
multiline
text block
```
````

## Core rules

- Lowercase `$name` stores a mutable variable.
- Uppercase `$NAME` stores an immutable constant; mixed-case names are rejected.
- `->` binds an operation result into a variable.
- Commas separate normal argument lists.
- Blocks close with `end`.
- List indexes start at `1`.
- `command <program> <args...>` runs directly without shell parsing; use bare words, quoted argument atoms, or variables as argv arguments. `command ... -> $out` captures stdout, while three captures (`$out, $err, $code`) also capture stderr and exit code.
- `shell "..."` is the explicit, shell/platform-dependent escape hatch for pipelines and redirections. The old `command "..."` syntax was removed.
- `arg`, `hasArg`, and `args` read command-line arguments passed after the script path.
- `"..."` is for one-line text; triple backticks are for multiline text.
- `choose "Question" ["A", "B"] -> $choice` prints an interactive menu and stores the chosen value.
- `log "identifier"` wraps a block with structured file logging in script-relative `logs/`.
- `exists` and `contains` provide built-in boolean predicates.
- Standard helpers transform text, lists, values, path strings, time, and random numbers; see [helpers.md](helpers.md) and the [date formatting guide](date-formatting.md).
- File commands (`create`, `copy`, `move`, `rename`, `remove`, `read`, `write`, `append`, `isFile`, `isDir`, `size`, `tempFile`, `tempDir`) and `files "pattern" -> $items` use script-relative paths where applicable.
- `and`, `or`, `not`, and condition parentheses compose conditions; priority is `not`, then `and`, then `or`.
- `case VALUE` / `when VALUE` / optional `else` dispatches exact value matches; see [conditionals.md](conditionals.md).
- `defer ... end` registers cleanup for scope exit; see [deferred cleanup](defer.md).

## Reference

| Topic | Guide |
| --- | --- |
| Comments | [comments.md](comments.md) |
| Variables | [variables.md](variables.md) |
| Constants | [constants.md](constants.md) |
| Strings and interpolation | [strings.md](strings.md) |
| Numeric expressions | [numeric-expressions.md](numeric-expressions.md) |
| Lists | [lists.md](lists.md) |
| Standard helpers | [helpers.md](helpers.md) · [Date formatting](date-formatting.md) |
| Logging | [logging.md](logging.md) |
| Files | [files.md](files.md) |
| Commands and input | [commands.md](commands.md) |
| Choose menus | [choose.md](choose.md) |
| Exit | [exit.md](exit.md) |
| Error handling | [errors.md](errors.md) |
| Functions | [functions.md](functions.md) |
| Conditionals | [conditionals.md](conditionals.md) |
| Predicates | [predicates.md](predicates.md) |
| Repeat loops | [repeat.md](repeat.md) |
| While loops | [while.md](while.md) |
| For loops | [for.md](for.md) |
| Modules | [modules.md](modules.md) |
| Project structure | [project-structure.md](project-structure.md) |

## Check and run

With no path, `shelly check` reads `Shelly.toml`, resolves `[project].entry`, and validates the manifest entry without running it or executing its commands. Checking also walks every reachable `import` and `include`, so missing dependencies, transitive-file errors, and import/include cycles are reported without executing any file:

```sh
shelly check
```

You can validate an explicit script path instead:

```sh
shelly check path/to/script.shelly
```

Editor integrations can request JSON diagnostics for an explicit path:

```sh
shelly check --format json path/to/script.shelly
```

JSON output is always an array of diagnostic objects with `file`, `line`, `column`, `severity`, and `message` fields. A successful check prints `[]`; diagnostics produce a failing check status. This validates the existing module declarations only—`check` introduces no new Shelly syntax.

Run a script:

```sh
shelly path/to/script.shelly Ada --dry-run
```

Inside that script, read those arguments with:

```shelly
arg 1 -> $name
hasArg "--dry-run" -> $dryrun
args -> $all
```

From source:

```sh
cargo run -- path/to/script.shelly
```

## Not implemented yet

- Nested lists.
- Ordered string filters.
- Multi-index, range, or filter mutation targets.
- Exporting normal variables.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Dictionaries

Use `{}` for keyed data. Simple keys can be identifiers; other keys use quoted strings.

```shelly
$user = { name: "Ada", "first-name": "Ada" }
shellout $user.name
shellout $user["first-name"]
```

Dot and bracket access can also be assigned. See [dictionaries](dictionaries.md).

## Nested values and JSON

Shelly values are scalars, lists, and dictionaries. Lists may contain dictionaries or other lists, so access chains can be composed:

```shelly
$data = { users: [{ name: "Ada" }], matrix: [[1, 2]] }
shellout $data.users[1].name
shellout "cell=$data.matrix[1][2]"
```

JSON conversion uses `jsonParse $text -> $value` and `jsonStringify $value -> $text`; see [`json.md`](json.md).
