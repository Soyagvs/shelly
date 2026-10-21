# Deferred cleanup

`defer` registers a block to run when the surrounding scope ends. Registration happens when execution reaches the statement; the block does not run immediately.

## Syntax

```shelly
defer
    shellout "cleanup"
end
```

`defer` takes no arguments and requires a block. Forms such as `defer cleanup` are rejected. A deferred block has no result. It shares the enclosing scope, so it can read and update values there—and its updates are visible to later code in that scope.

## Order and scope

Deferred blocks run in **last-in, first-out (LIFO)** order, once per enclosing scope.

```shelly
defer
    shellout "first registered, last run"
end

defer
    shellout "second registered, first run"
end
```

This prints `second registered, first run`, then `first registered, last run`.

Each block belongs to the nearest scope and does not escape it. Nested scopes clean up from the inside out. A deferred block is executed as its own scope, so a `defer` inside cleanup is also run when that cleanup block ends.

## Every exit path

The current implementation runs deferred blocks when a scope ends normally or with an ordinary error. It also unwinds them for Shelly’s encoded control signals: `exit`, `return`, `break`, and `continue`.

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

`defer` complements [`try`/`catch`](errors.md): use `catch` to recover from an error, and `defer` for work that must happen regardless. In the implementation, control signals are not caught by `try`, but they still trigger deferred cleanup. See [`exit`](exit.md) for process termination behavior.

## Cleanup errors and outcomes

Cleanup is attempted in LIFO order. If one deferred block fails, later deferred blocks are skipped. If the scope already had an ordinary error or control signal, that earlier outcome remains authoritative; the cleanup error does not replace it. If the scope would otherwise succeed, the first cleanup error makes it fail.

Keep deferred blocks small and reliable. They run only when their scope unwinds, so many registrations, expensive cleanup, or cleanup that itself fails can make control flow harder to follow—and can prevent remaining cleanup from running.

## Next steps

- [Error handling](errors.md) for recoverable failures and control signals.
- [Exit](exit.md) for explicit process termination.
- [Language Guide](language.md) for the syntax overview.
- [Documentation index](index.md) for the book-like reference map.
