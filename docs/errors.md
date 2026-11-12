# Recover from ordinary Shelly errors

Use `try`, a required `catch`, and `end` when a runtime failure is expected and recoverable. Use [defer](defer.md) when cleanup must run regardless of the outcome.

## 🚀 Quick path

```shelly
try -> $error
    command false
catch
    shellout "failed: $error.message"
end
```

If the protected block succeeds, Shelly skips `catch` and continues after `end`. An ordinary error runs `catch`, then execution continues.

## 📖 Syntax and rules

| Form | Meaning |
| --- | --- |
| `try` | Catch an error without storing it |
| `try -> $error` | Store the structured error dictionary |
| `catch` | Required handler with no condition or destination |
| `end` | Closes the complete block |

- Every `try` has exactly one `catch`.
- Captured errors have `kind` and `message`.
- Try and catch share the surrounding scope; nested tries use their nearest matching catch.
- Supported headers are exactly `try` and `try -> $variable`.
- `try $error`, `try as $error`, `catch $error`, conditional catches, and `finally` are rejected.

```shelly
try
    include "optional.shelly"
catch
    shellout "optional setup failed"
end
```

## 🔎 Structured errors and retries

Process failures also include `program`, `args`, `code`, `stdout`, and `stderr`. Timeouts have `kind == "timeout"` and include `program` and `args`, but no process exit code or completed output. HTTP failures use `kind` and `message`; network, policy, syntax, and limit failures do not expose a response dictionary. `menu` reports `menu cancelled` for TTY Escape cancellation and `menu input closed` when non-TTY stdin reaches EOF. The removed `choose` command produces a migration diagnostic directing scripts to a `menu` block.

A response with status 400 or higher has `kind == "http"` and intentionally redacts status, body, headers, and URL. Secret lookup failures include invalid environment names, missing or empty values, and non-text environment values.

`retry <attempts>` preserves the final structured error and retries only known timeout failures. Control signals and ordinary command failures are never retried.

```shelly
try -> $error
    command false
catch
    if $error.kind == "process" and $error.code != 0
        shellout "command failed: $error.message"
    end
end
```

See [HTTP requests](http.md), [Timeouts](timeouts.md), [Commands and input](commands.md), and [Environment-only secrets](secrets.md) for subsystem details.

## 🧩 Nested tries and control signals

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

The inner handler catches the command failure, so the outer handler does not run. An unhandled ordinary error propagates outward.

`exit`, `break`, `continue`, and function `return` are control signals, not recoverable errors, so `try` does not catch them:

```shelly
try
    exit 1
catch
    shellout "this will not run"
end
```

Use [defer](defer.md) for cleanup on every exit path and [exit](exit.md) for process termination.

## 🛠️ Audit finalization

Dry-run audit-log writes and finalization with `sync_all()` can fail. Terminal records are written before synchronization, so a synchronization failure exits with an error even when a terminal `complete` record exists. `sync_all()` cannot guarantee durability against OS, filesystem, storage-device, or hardware failures.

## Related guides

- [Deferred cleanup](defer.md) — run cleanup during unwinding.
- [Exit](exit.md) — terminate the whole process.
- [Timeouts](timeouts.md) — understand deadline failures.
