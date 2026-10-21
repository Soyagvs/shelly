# Exit

[Documentation index](index.md) · [Error handling](errors.md) · [Deferred cleanup](defer.md)

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

`exit` terminates the whole Shelly process and sets its process exit status. Any [`defer`](defer.md) blocks in active scopes unwind first; cleanup does not turn an explicit control signal into an ordinary error.

```shelly
exit
exit 0
exit 1
```

## Rules

- `exit` is the same as `exit 0`; zero reports success.
- A non-zero status reports failure. `1` is the general failure status.
- The form is `exit` or `exit <expression>`: at most one expression is allowed.
- The expression must evaluate to an integer from `0` through `255`.
- Literals, numeric expressions, variables, and constants are valid expressions.
- `exit` propagates past functions and enclosing blocks; it stops the whole script, not only the current scope.

These forms are rejected:

```shelly
exit -1       // outside the valid range
exit 256      // outside the valid range
exit "1"      // not an integer
exit 1 2      // more than one argument
```

## Expressions

```shelly
$code = 0
exit $code
```

```shelly
$BASE = 1
exit $BASE + 1
```

## Inside blocks

```shelly
repeat 3
    shellout "before exit"
    exit 0
end

shellout "never runs"
```

`try` does not catch `exit`; it is a control signal, not a recoverable error:

```shelly
try
    exit 1
catch
    shellout "this will not run"
end
```

Deferred blocks still run while `exit` unwinds their active scopes. See [`defer`](defer.md) for cleanup order and failure behavior.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
