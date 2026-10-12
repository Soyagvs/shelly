# Functions

Functions define reusable blocks with `fn` and `end`.

```ly
fn greet $name, $prefix = "Hola"
    shellout "$prefix $name"
end

greet "Ada"
greet "Ada", "Hello"
```

## Definitions and calls

- Function definitions start with `fn` and close with `end`.
- Functions are called by name.
- Definitions use comma-separated parameters.
- Calls use comma-separated arguments.
- Arguments may be literal strings, numbers, booleans, variables, or constants.
- Parameters use `$` variable syntax and normal identifier rules.

## Default parameters

```ly
fn greet $name, $prefix = "Hola"
    shellout "$prefix $name"
end
```

Defaulted parameters must come after required parameters. A required parameter after a default is an error.

## Scope and `global`

Function scope is local by default. Assignments inside a function do not mutate outer variables.

```ly
$outside = "outer"
$$APP = "Shelly"

fn show $name
    global $outside, $$APP
    shellout "$name from $$APP with $outside"
end

show "Ada"
```

Rules:

- Outer variables and constants require `global` declarations before use inside a function.
- `global` supports comma-separated identifiers: `global $name, $age` and `global $$NAME, $$AGE`.
- Initial `global` behavior is read-only for outer values: globals can be read inside a function, but assignments remain local.
- Constants remain immutable.
- Functions do not return values yet.
