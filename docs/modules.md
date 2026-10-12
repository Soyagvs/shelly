# Modules: import, include, and export

Shelly can split scripts across files with `import`, `include`, and `export`.

## Export

Only constants and functions can be exported.

```shelly
export $$GREETING = "Hola"

export fn greet $name
    global $$GREETING
    shellout "$$GREETING $name"
end
```

Normal variables are not exportable yet. `export $name = "x"` is rejected.

## Import

`import` loads only exported constants and functions from another file.

```shelly
import "examples/module.shelly"
greet "Ada"
shellout $$GREETING
```

Private top-level code in the imported file does not run.

## Include

`include` executes the whole target file in the current scope, like copying it at that point.

```shelly
include "examples/setup.shelly"
shellout $color
```

## Paths and cycles

- Import/include paths are relative to the file that declares them.
- Missing files are line-numbered errors.
- Import/include cycles are detected and reported.
- Duplicate imports are safe.
