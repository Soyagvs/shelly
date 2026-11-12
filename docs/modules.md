# Organize Shelly programs across files

Use `import` to load exported definitions, `include` to execute a complete file, and `export` to publish constants or functions.

## 🚀 Quick path

```shelly
import "examples/module.shelly"
include "examples/setup.shelly"
```

Choose `import` for reusable definitions. Choose `include` when the target file should run in the current scope.

## 📖 Import, include, and export

| Statement | Loads | Runs private top-level code | Scope behavior |
| --- | --- | --- | --- |
| `import "file.shelly"` | Exported constants and functions | No | Names are available directly |
| `include "file.shelly"` | The complete file | Yes | Executes in the current scope |
| `export ...` | A constant or function for importers | N/A | Normal variables cannot be exported |

```shelly
# lib/greetings.shelly
export $GREETING = "Hola"

export fn greet($name) {
    global $GREETING
    shellout "$GREETING $name"
}
```

```shelly
# main.shelly
import "lib/greetings.shelly"
call greet("Ada")
shellout $GREETING
```

Imported and included names are shared directly; Shelly has no module namespace prefix. Duplicate imports are ignored after the first successful import. Includes are not cached, so each `include` executes its target again.

## 🧭 Paths and scope

The path must be a string literal and is resolved relative to the file containing the statement. Relative paths, `.` and `..` components, and absolute paths are accepted. Shelly canonicalizes the resolved path, so equivalent spellings refer to one module and duplicate imports are recognized.

These forms are invalid: `import $path`, `include helpers.shelly`, and `import "one.shelly" extra`. Missing or unreadable files are errors.

Import/include cycles are errors. A valid declaration is, for example, `import "lib/helpers.shelly"` or `include "setup.shelly"`.

## 🛠️ Module graph checking

`shelly check` validates the complete literal `import`/`include` graph without executing the script or dependencies. It parses and checks reachable files and reports:

- missing, invalid, or unreadable dependencies;
- diagnostics from transitive dependencies; and
- cycles with the dependency chain that forms the cycle.

Computed or variable paths cannot be resolved by graph checking. Use `--format json` for a JSON array of diagnostics. Each object contains `file`, `line`, `column`, `severity`, and `message`; an empty array means the graph passed, while any diagnostic makes `check` fail.

## 🎨 Shared style constants

The optional `lib/palette.shelly` module exports semantic style descriptors such as `$SUCCESS`, `$WARNING`, and `$ERROR` for `style(...)`. They are ordinary exported constants, not native aliases:

```shelly
import "lib/palette.shelly"
shellout style($SUCCESS, "done")
```

See [Terminal styling](terminal-styling.md) for the complete vocabulary and color policy.

## Related guides

- [Functions](functions.md) — define and export reusable behavior.
- [Project structure](project-structure.md) — arrange a Shelly project.
- [Language overview](language.md) — see the complete syntax map.
