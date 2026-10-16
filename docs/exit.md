# Exit

[Documentation index](index.md) · [Error handling](errors.md) · [Deferred cleanup](defer.md)

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

`exit` stops the whole Shelly process immediately. Any [`defer`](defer.md) blocks in active scopes run first; cleanup does not turn an explicit control signal into an ordinary error.

```shelly
exit
exit 0
exit 1
```

## Rules

- `exit` is the same as `exit 0`.
- `exit 0` means success.
- `exit 1` means general failure.
- Valid codes are integers from `0` to `255`.
- Codes may be literals, numeric expressions, variables, or constants.
- `exit` stops the whole script, not only the current block.

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

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
