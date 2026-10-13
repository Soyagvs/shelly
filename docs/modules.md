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
- Import/include cycles are errors.
- Duplicate imports are ignored after the first successful import.

## Export

```shelly
export $$GREETING = "Hola"

export fn greet $name
    global $$GREETING
    shellout "$$GREETING $name"
end
```

## Import

```shelly
import "examples/module.shelly"

greet "Ada"
shellout $$GREETING
```

## Include

```shelly
include "examples/setup.shelly"

shellout $color
```

Use `import` for reusable modules. Use `include` when you intentionally want the whole file to run.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
