# Build reusable Shelly functions

Functions package behavior into named, reusable blocks. Define them with `fn`, invoke them with `call`, and optionally return one scalar value.

## 🚀 Quick path

```shelly
fn greet($name, $prefix = "Hola") {
    shellout "$prefix $name"
}

call greet("Ada")
call greet("Ada", "Hello")
```

## 📖 Syntax and rules

```shelly
fn name($required, $optional = "default") {
    return $optional
}

call name("value") -> $result
```

- Definitions start with `fn name(...) {` and close with `}`.
- Calls use `call name(args)`; arguments are positional and comma-separated.
- Use `fn name() {` and `call name()` when there are no parameters.
- Parameters use `$` variable syntax. Required parameters must precede defaulted parameters.
- Default values apply when the argument is omitted.
- A function has local scope by default. Parameters and values copied from outer scope are independent values.
- `return value` exits the current function and returns one scalar value.
- Capture a return with `call function(args) -> $result`; without `->`, a return value is discarded.
- A call with `->` must receive a return value. Reaching `}` without `return` is an error for such a call.

## 🔎 Scope, globals, and recursion

Variables assigned in a function are local, even if an outer variable has the same name. `global` copies selected outer values into the function; it does not create a writable alias. Mutating a copied collection or assigning the name does not update the outer value. Copied constants remain immutable.

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

A declaration may list several existing identifiers: `global $APP, $MODE`. It must be inside a function and cannot be empty.

Functions may call themselves. Each call receives a new local scope:

```shelly
fn countdown($n) {
    if $n == 0
        return "done"
    end
    shellout $n
    call countdown($n - 1)
}

call countdown(3)
```

## ↩️ Return values and control flow

```shelly
fn label($name) {
    return "hello $name"
}

call label("Ada") -> $message
shellout $message
```

The destination after `->` must be one mutable variable, not a constant, path, or expression. `return` affects only the current function. `break` and `continue` affect the nearest loop in that function and cannot cross a function boundary. A control signal outside its valid function or loop context is an error; control signals are not converted into caught errors.

## ⚠️ Rejected syntax and errors

The checker rejects duplicate names, missing braces, missing or extra arguments, a required parameter after a default parameter, invalid parameter or argument syntax, empty or unknown `global` declarations, text after a global identifier, unknown function calls, and invalid return destinations. `return` outside a function is also invalid.

## 📦 Export functions from modules

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

`import` loads exported functions and constants without running private top-level code. See [Modules](modules.md) for import and include behavior.

## Related guides

- [Modules](modules.md) — share functions across files.
- [Error handling](errors.md) — recover from ordinary runtime errors.
- [Deferred cleanup](defer.md) — clean up when function scopes unwind.
- [Control flow](conditionals.md) — branch inside a function.
