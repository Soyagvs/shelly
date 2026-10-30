# Timeouts

Shelly provides explicit deadlines for commands and blocks. Timeouts are not enabled automatically: a command or block runs without a deadline unless you add one.

## Per-process deadlines

Add `timeout SECONDS` after a command's arguments to limit that process:

```shelly
command curl https://example.com timeout 10
```

`SECONDS` must be a finite, positive number; fractional values such as `0.5` are valid. Zero, negative, non-numeric, infinite, and non-finite values are rejected. The deadline applies to that process only.

A command deadline covers both output modes: it bounds a streaming command and a command that captures stdout (or stdout, stderr, and the exit code). On timeout, the command is interrupted and raises `kind == "timeout"`; the timeout error includes `program` and `args`, but has no process exit code or completed-output fields.

## Inherited deadlines

Put `timeout SECONDS` before a block and close it with `end` to set a total deadline for everything inside:

```shelly
timeout 30
    call fetchData
    for $item in $items
        process $item
    end
end
```

The deadline is inherited through `call`, functions, and loops. It is checked at statement boundaries and while waiting for processes; it is not an arbitrary interruption mechanism for every blocking operation. When a deadline has expired, subsequent statements—including deferred cleanup statements—may not start. No grace period or hard wall-clock guarantee is provided. A nested `timeout` block uses the shorter of its deadline and the inherited block deadline. A command-level `timeout SECONDS` establishes its own effective command deadline and can override the inherited block deadline. A block timeout reports `kind == "timeout"` without process metadata; a process timeout reports `program` and `args` as well.

## Handling timeouts

A timed-out operation raises a catchable error whose `kind` is `"timeout"`. Process timeouts also include `program` and `args`; block timeouts do not, because no particular process is necessarily responsible:

```shelly
try -> $error
    command slow-program timeout 2
catch
    if $error.kind == "timeout"
        shellout "the operation timed out"
    end
end
```

Use `try`/`catch` when a timeout is expected and recoverable; see [Error handling](errors.md). For command syntax and capture behavior, see [Commands and input](commands.md). HTTP requests also have a per-request timeout (30 seconds by default, up to 120 seconds); see [HTTP requests](http.md).
