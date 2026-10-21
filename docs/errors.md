# Error handling

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

[Documentation index](index.md) · [Deferred cleanup](defer.md) · [Timeouts](timeouts.md) · [Exit](exit.md)

Use `try`, a required `catch`, and `end` to handle ordinary runtime errors.

```shelly
try -> $error
    command false
catch
    shellout "failed: $error.message"
end
```

## Syntax and rules

- `try` starts a protected block. You may omit the destination when you do not need to inspect the error.
- `try -> $error` stores a structured error dictionary before the catch body runs. Every error has `kind` and `message`.
- `catch` is required exactly once for every `try`. It takes no condition or destination; inspect the captured dictionary in its body.
- `end` closes the whole `try`/`catch` block.
- If the try body succeeds, Shelly skips the catch body. An ordinary error runs the catch body, then execution continues after `end`.
- Try blocks do not create a new scope. The try body and catch body share the surrounding scope, and nested tries use their nearest matching `catch`.
- The supported headers are exactly `try` and `try -> $variable`. Forms such as `try $error`, `try as $error`, or a non-variable destination are rejected. `catch $error`, conditional catches, and `finally` are not supported.

Without a destination, errors are still caught but are not available through a variable:

```shelly
try
    include "optional.shelly"
catch
    shellout "optional setup failed"
end
```

## Structured errors and timeouts

Process failures include `program`, `args`, `code`, `stdout`, and `stderr` in addition to `kind` and `message`. A timeout is a distinct error with `kind` set to `"timeout"`; it includes `program` and `args`, but does not represent a process exit code or completed output. See [Timeouts](timeouts.md) for command and block deadlines, and [Commands and input](commands.md) for process and capture behavior.

```shelly
try -> $error
    command false
catch
    if $error.kind == "process" and $error.code != 0
        shellout "command failed: $error.message"
    end
end
```

Use [`defer`](defer.md) when cleanup must run for success, errors, or control signals; use `try` when failure is expected and recoverable.

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

The inner `catch` handles the command failure, so the outer `catch` does not run. If the inner block does not handle an ordinary error, it propagates to the enclosing `try`.

## Control signals are not caught

```shelly
try
    exit 1
catch
    shellout "this will not run"
end
```

`exit`, `break`, `continue`, and function `return` are control signals, not recoverable errors, so `try` does not catch them. Use [`defer`](defer.md) for cleanup on every exit path; see [`exit`](exit.md) for process termination behavior.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
