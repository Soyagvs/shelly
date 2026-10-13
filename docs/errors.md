# Error handling

Use `try`, required `catch`, and `end` to handle ordinary runtime errors.

```shelly
try -> $error
    command "false"
catch
    shellout "failed: $error"
end
```

## Rules

- `try` starts a protected block.
- `try -> $error` stores the ordinary error message before the catch body runs.
- `catch` is required exactly once per `try`.
- `end` closes the block.
- If the try body succeeds, Shelly skips the catch body.
- If the try body returns an ordinary error, Shelly runs the catch body in the same scope.
- Nested `try` blocks are supported.
- Control signals are not caught: `exit`, `break`, `continue`, and function `return` keep their normal behavior.

Use `try` around commands or statements where failure is expected and recoverable.

## Nested try

```shelly
try -> $outerError
    try -> $innerError
        command "false"
    catch
        shellout "inner failed: $innerError"
    end
catch
    shellout "outer failed: $outerError"
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
