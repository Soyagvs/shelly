# Bound commands and blocks with timeouts

Timeouts are opt-in deadlines. A command or block runs without a deadline unless you add `timeout`.

## Quick path

```shelly
command curl https://example.com timeout 10

timeout 30
    call fetchData
    process $item
end
```

## Command deadlines

`timeout SECONDS` after a command limits that process. The value must be finite and positive; fractional values are valid. Zero, negative, non-numeric, infinite, and non-finite values fail.

The deadline covers streaming and captured output. A timed-out process is interrupted and raises `kind == "timeout"`; its error includes `program` and `args`, but no exit code or completed output.

## Block deadlines

A timeout block sets one inherited deadline for statements, `call`, functions, loops, and process waits. It is checked at statement boundaries and while waiting for processes, not as an arbitrary interrupt for every blocking operation. A nested block uses the shorter inherited or nested deadline. A command-level timeout establishes its own effective command deadline and can override the inherited block deadline.

When a block deadline expires, later statements—including deferred cleanup—may not start. There is no grace period or hard wall-clock guarantee. Block timeout errors have `kind == "timeout"` without process metadata; process timeout errors include `program` and `args`.

## Handle expiration

```shelly
try -> $error
    command slow-program timeout 2
catch
    if $error.kind == "timeout"
        shellout "the operation timed out"
    end
end
```

Use [Error handling](errors.md) for recovery. HTTP has its own per-request timeout: 30 seconds by default and at most 120 seconds; see [HTTP requests](http.md).
