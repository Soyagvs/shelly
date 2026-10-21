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
- Parameters use `$` variable syntax and arguments are positional.
- Definitions and calls use comma-separated values inside parentheses.
- Default parameters must come after all required parameters. A default is used when that argument is omitted.
- Function scope is local by default; values passed to parameters and values copied from outer scope are independent values.
- Use `global` to copy selected outer variables or constants into the function. It does not create a writable alias.
- `return value` exits the current function early and returns one scalar value.
- Capture a returned value with `call function(args) -> $result`.
- Calling a returning function without `->` is allowed and discards the result.
- A call with `->` must receive a return value; a function that reaches its closing brace without returning is an error.

## Local scope and global copies

Variables created or assigned in a function are local, even when an outer variable has the same name. `global` explicitly copies an outer value into the function. Reassigning the name, or mutating a copied collection, does not update the outer variable. Constants copied with `global` remain immutable.

```shelly
$outside = "outer"

fn demo() {
    global $outside
    $outside = "inner copy"
    shellout $outside
}

call demo()
shellout $outside
# outer
```

A global declaration may list several identifiers (`global $APP, $MODE`) and every identifier must already exist in the outer scope. Globals are declarations inside a function, not call arguments.

## Defaults and recursion

Required parameters must precede defaulted parameters:

```shelly
fn greet($name, $prefix = "Hola") {
    shellout "$prefix $name"
}
```

Functions may call themselves. Each call gets a new local scope and its own parameter values, so recursion stops only when the function's condition returns or otherwise exits.

```shelly
fn countdown($n) {
    if $n == 0
        return "done"
    end
    shellout $n
    # A recursive call is an ordinary function call.
    call countdown($n - 1)
}

call countdown(3)
```

## Return values and capture

`return` exits the current function immediately. The value is captured with the same arrow binding used by other operations:

```shelly
fn label($name) {
    return "hello $name"
}

call label("Ada") -> $message
shellout $message
```

A return can be ignored when the call is used for its effects:

```shelly
call label("Ada")
```

The destination after `->` must be one variable. Capturing into a constant, a path, or any other expression is rejected. Capturing from a function that finishes without `return` is also rejected; without a destination, that same no-return function call is valid.

## Control-flow boundaries

- `return` affects only the current function. It may be used from inside a conditional or loop and skips the rest of that function.
- `break` exits the nearest enclosing `repeat`, `while`, or `for` loop in the current function.
- `continue` skips to the next iteration of the nearest enclosing loop in the current function.
- `break` and `continue` cannot cross a function boundary to control a caller's loop. If the function returns with either signal and no loop exists inside that function, the call fails with an outside-loop error.
- `return` outside a function is an error. Control signals are not converted into ordinary caught errors.

## Rejected syntax and errors

The checker rejects malformed definitions and calls, including duplicate function names, missing closing braces, missing or extra arguments, a required parameter after a default parameter, and invalid parameter or argument syntax. It also rejects an empty `global` declaration, unknown globals, and text after a global identifier. A function call must name a known function, and a return destination must be a variable.

## Global access

```shelly
$APP = "Shelly"

fn greet($name) {
    global $APP
    shellout "hello $name from $APP"
}

call greet("Ada")
```

## Export and import

Functions can be shared from a module with `export fn`; another file receives them with `import`:

```shelly
# functions/greetings.shelly
export fn greet($name) {
    return "hello $name"
}
```

```shelly
# main.shelly
import "functions/greetings.shelly"
call greet("Ada") -> $message
shellout $message
```

`import` loads exported functions (and exported constants) without running private top-level code in the imported file. Normal variables cannot be exported. Use [`modules.md`](modules.md) for the complete import/include behavior.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
