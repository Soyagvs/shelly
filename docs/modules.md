# Modules

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Shelly can split scripts across files with `import`, `include`, and `export`.

```shelly
import "examples/module.shelly"
include "examples/setup.shelly"
```

## Rules

- `export` exposes constants and functions.
- Normal variables are not exportable yet.
- `import` loads only exported constants and functions.
- Private top-level code in an imported file does not run.
- `include` executes the whole target file in the current scope.
- Imported and included names are shared directly; modules have no namespace, so use the names themselves rather than a module prefix.
- Import/include cycles are errors.
- Duplicate `import`s are ignored after the first successful import. `include` is not cached: each include executes the target again.

## Paths and scope

The path must be a string literal, and it is resolved relative to the file containing the statement. Relative paths such as `"../shared.shelly"` and paths with `.` or `..` components are valid. Shelly canonicalizes the resolved path before loading it, so different relative spellings of the same file refer to one module and duplicate imports are recognized. An absolute path is also accepted.

The two forms deliberately have different scope behavior:

- `import "path.shelly"` reads the file's exported constants and functions into the current script. It does not run private top-level statements.
- `include "path.shelly"` runs the complete file in the current scope. Variables and other effects are visible as though the file's contents occurred at the include site.

A valid declaration is, for example, `import "lib/helpers.shelly"` or `include "setup.shelly"`. A variable, expression, bare word, or extra text is rejected: `import $path`, `include helpers.shelly`, and `import "one.shelly" extra` are invalid. Missing or unreadable files are errors too.

## Export

```shelly
export $GREETING = "Hola"

export fn greet($name) {
    global $GREETING
    shellout "$GREETING $name"
}
```

## Import

```shelly
import "examples/module.shelly"

call greet("Ada")
shellout $GREETING
```

## Include

```shelly
include "examples/setup.shelly"

shellout $color
```

Use `import` for reusable modules. Use `include` when you intentionally want the whole file to run.

## Module graph checking

`shelly check` validates the complete `import`/`include` graph reachable from the checked script without executing the script or any dependency. It follows only literal `import` and `include` declarations; it cannot resolve a computed or variable path. It parses and checks each reachable file, then reports:

- missing or invalid dependencies, including paths that cannot be read;
- diagnostics from transitive dependencies, not only direct imports/includes; and
- import/include cycles, with the dependency chain that forms the cycle.

Graph checking does not add or change Shelly syntax: existing `import` and `include` statements are the only dependency declarations.

Use `--format json` for machine-readable diagnostics. The command prints a JSON array; each diagnostic is an object with `file`, `line`, `column`, `severity`, and `message` fields. An empty array means the graph passed, while any diagnostic still makes `check` fail.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
