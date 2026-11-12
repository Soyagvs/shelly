# Stop a Shelly process with an exit status

`exit` terminates the whole Shelly process and sets its process exit status. Active [defer](defer.md) blocks unwind first; cleanup does not turn the control signal into an ordinary error.

## 🚀 Quick path

```shelly
exit       # success, equivalent to exit 0
exit 1     # general failure
```

## 📖 Syntax and status rules

The form is `exit` or `exit <expression>`, with at most one expression.

- No expression is the same as `exit 0`; zero reports success.
- A non-zero status reports failure; `1` is the general failure status.
- The expression must evaluate to an integer from `0` through `255`.
- Literals, numeric expressions, variables, and constants are valid.
- `exit` propagates through functions and enclosing blocks and stops the whole script.

```shelly
$code = 0
exit $code

$BASE = 1
exit $BASE + 1
```

## ⚠️ Rejected forms

```shelly
exit -1       # outside the valid range
exit 256      # outside the valid range
exit "1"      # not an integer
exit 1 2      # more than one argument
```

## 🧩 Inside blocks and error handling

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

Deferred blocks still run while `exit` unwinds their active scopes. See [Deferred cleanup](defer.md) for cleanup order and failure behavior.

## Related guides

- [Error handling](errors.md) — handle ordinary runtime errors.
- [Deferred cleanup](defer.md) — run cleanup on termination.
- [Language overview](language.md) — review control-flow syntax.
