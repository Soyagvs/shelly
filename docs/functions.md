# Functions

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Functions define reusable blocks with `fn name(...) {` and `}`.

```shelly
fn greet($name, $prefix = "Hola") {
    shellout "$prefix $name"
}

call greet("Ada")
call greet("Ada", "Hello")
```

## Rules

- Function definitions start with `fn name(...) {` and close with `}`.
- Functions are called with `call name(args)`.
- Use `fn name() {` and `call name()` when there are no parameters.
- Parameters use `$` variable syntax.
- Definitions and calls use comma-separated values inside parentheses.
- Default parameters must come after required parameters.
- Function scope is local by default.
- Use `global` to read selected outer values.
- `return value` exits the function early.
- Capture a returned value with `call function(args) -> $result`.
- Calling a returning function without `->` discards the result.

## Local scope

```shelly
$outside = "outer"

fn demo() {
    $outside = "inner"
    shellout $outside
}

call demo()
shellout $outside
```

## Return values

```shelly
fn label($name) {
    return "hello $name"
}

call label("Ada") -> $message
shellout $message
```

## Global access

```shelly
$$APP = "Shelly"

fn greet($name) {
    global $$APP
    shellout "hello $name from $$APP"
}

call greet("Ada")
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
