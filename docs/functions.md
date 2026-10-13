# Functions

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Functions define reusable blocks with `fn` and `end`.

```shelly
fn greet $name, $prefix = "Hola"
    shellout "$prefix $name"
end

greet "Ada"
greet "Ada", "Hello"
```

## Rules

- Function definitions start with `fn` and close with `end`.
- Functions are called by name.
- Parameters use `$` variable syntax.
- Definitions and calls use comma-separated values.
- Default parameters must come after required parameters.
- Function scope is local by default.
- Use `global` to read or write selected outer values.
- Functions do not return values yet.

## Local scope

```shelly
$outside = "outer"

fn demo
    $outside = "inner"
    shellout $outside
end

demo
shellout $outside
```

## Global access

```shelly
$$APP = "Shelly"

fn greet $name
    global $$APP
    shellout "hello $name from $$APP"
end
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
