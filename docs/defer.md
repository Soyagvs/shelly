# Guarantee cleanup while Shelly scopes unwind

`defer` registers a block to run when the surrounding scope ends. Registration happens when execution reaches the statement; the block does not run immediately.

## 🚀 Quick path

```shelly
defer
    shellout "cleanup"
end
```

## 📖 Syntax and scope

`defer` takes no arguments and requires a block. `defer cleanup` is invalid. A deferred block has no result, shares the enclosing scope, and can read or update values visible there.

Deferred blocks run once per enclosing scope in **last-in, first-out (LIFO)** order:

```shelly
defer
    shellout "first registered, last run"
end

defer
    shellout "second registered, first run"
end
```

Each block belongs to its nearest scope. Nested scopes clean up from the inside out. A deferred block executes as its own scope, so a `defer` registered during cleanup runs when that cleanup block ends.

## 🔁 Every exit path

Deferred blocks run when a scope ends normally or with an ordinary error. Shelly also unwinds them for `exit`, `return`, `break`, and `continue`.

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

An already-expired active deadline can prevent a deferred statement from starting. Outer cleanup also requires its enclosing deadline to remain valid, so cleanup is not a hard wall-clock guarantee.

`defer` complements [`try`/`catch`](errors.md): use `catch` to recover from an error and `defer` for work that must happen regardless. Control signals are not caught by `try`, but they still trigger deferred cleanup. See [Exit](exit.md) for process termination.

## ⚠️ Cleanup errors and outcomes

Cleanup is attempted in LIFO order. If one deferred block fails, later deferred blocks are skipped.

| Scope outcome | Result when cleanup fails |
| --- | --- |
| Already had an error or control signal | The earlier outcome remains authoritative |
| Would otherwise succeed | The first cleanup error makes the scope fail |

Keep deferred blocks small and reliable. Many registrations, expensive cleanup, or cleanup that fails can make control flow harder to follow and prevent remaining cleanup from running.

## Related guides

- [Error handling](errors.md) — recover from ordinary failures.
- [Exit](exit.md) — terminate the process and unwind cleanup.
- [Functions](functions.md) — understand function scope boundaries.
- [Language overview](language.md) — review Shelly syntax.
