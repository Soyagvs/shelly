# Timeouts

Shelly provides explicit deadlines for commands and blocks. Timeouts are not enabled automatically: a command or block runs without a deadline unless you add one.

## Per-process deadlines

Add `timeout SECONDS` after a command's arguments to limit that process:

```shelly
command curl https://example.com timeout 10
```

`SECONDS` must be a positive integer or fractional number. The deadline applies to that process only.

## Inherited deadlines

Put `timeout SECONDS` before a block and close it with `end` to set a total deadline for everything inside:

```shelly
timeout 30
    fetchData
    for $item in $items
        process $item
    end
end
```

The deadline is inherited through function calls and loops. A nested deadline may be shorter, but it cannot extend the deadline inherited from its parent.

## Handling timeouts

A timed-out operation raises a catchable error whose `kind` is `"timeout"`:

```shelly
try -> $error
    command slow-program timeout 2
catch
    if $error.kind == "timeout"
        shellout "the operation timed out"
    end
end
```

Use `try`/`catch` when a timeout is expected and recoverable; see [Error handling](errors.md). For command syntax and capture behavior, see [Commands and input](commands.md).
