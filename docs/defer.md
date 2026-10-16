# Deferred cleanup

`defer` registers a block to run when the surrounding scope ends. It is useful for cleanup that must happen whether work succeeds, fails, or leaves the scope with a control signal.

## Syntax

```shelly
defer
    shellout "cleanup"
end
```

A `defer` block has no result. It is registered where it appears, then runs when its enclosing scope exits. The block shares the enclosing scope, so it can read and update values available there.

## Order and scope

Deferred blocks run in **last-in, first-out (LIFO)** order. Each block belongs to the nearest enclosing scope and runs once when that scope exits.

```shelly
defer
    shellout "first registered, last run"
end

defer
    shellout "second registered, first run"
end
```

This prints `second registered, first run`, then `first registered, last run`.

Nested scopes clean up from the inside out:

```shelly
try
    defer
        shellout "inner cleanup"
    end
    command false
catch
    shellout "handled"
end
```

The inner cleanup runs as the protected scope exits. A deferred block does not escape its enclosing scope.

## Every exit path

Cleanup runs when a scope ends normally and when it ends because of an ordinary error. It also runs for Shelly control signals: `exit`, `return`, `break`, and `continue`.

```shelly
defer
    shellout "always runs"
end

try
    command false
catch
    shellout "the error was handled"
end
```

`defer` complements [`try`/`catch`](errors.md): use `catch` to recover from an error, and `defer` for work that must happen regardless. Control signals are not caught by `try`, but they still trigger deferred cleanup. See [`exit`](exit.md) for process termination behavior.

## Cleanup errors

If cleanup itself fails, Shelly preserves the original outcome. An earlier ordinary error or control signal remains authoritative rather than being replaced by a cleanup error. When the scope would otherwise succeed, a cleanup error makes the scope fail.

Keep deferred blocks small and reliable; if cleanup can fail, make that failure meaningful and avoid hiding the original problem.

## Next steps

- [Error handling](errors.md) for recoverable failures and control signals.
- [Exit](exit.md) for explicit process termination.
- [Language Guide](language.md) for the syntax overview.
- [Documentation index](index.md) for the book-like reference map.
