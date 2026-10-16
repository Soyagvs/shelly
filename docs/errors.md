# Error handling

Use `try`, required `catch`, and `end` to handle ordinary runtime errors.

```shelly
try -> $error
    command false
catch
    shellout "failed: $error.message"
end
```

## Rules

- `try` starts a protected block.
- `try -> $error` stores a structured error dictionary before the catch body runs. It has `kind` and `message` fields.
- Process errors also include `program`, `args`, `code`, `stdout`, and `stderr` fields. A deadline failure has `kind` set to `"timeout"`; see [Timeouts](timeouts.md).
- `catch` is required exactly once per `try`.
- `end` closes the block.
- If the try body succeeds, Shelly skips the catch body.
- If the try body returns an ordinary error, Shelly runs the catch body in the same scope.
- Nested `try` blocks are supported.
- Control signals are not caught: `exit`, `break`, `continue`, and function `return` keep their normal behavior.

Use `try` around commands or statements where failure is expected and recoverable. For example, inspect the error kind and process exit code in a catch:

```shelly
try -> $error
    command false
catch
    if $error.kind == "process" and $error.code != 0
        shellout "command failed: $error.message"
    end
end
```

## Nested try

```shelly
try -> $outererror
    try -> $innererror
        command false
    catch
        shellout "inner failed: $innererror.message"
    end
catch
    shellout "outer failed: $outererror.message"
end
```

The inner `catch` handles the `command` failure, so the outer `catch` does not run.

## Control signals are not caught

```shelly
try
    exit 1
catch
    shellout "this will not run"
end
```

`exit`, `break`, `continue`, and function `return` keep their normal behavior. They are not treated as recoverable errors.
