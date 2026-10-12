# Exit

`exit` terminates the whole Shelly process immediately.

```ly
exit
exit 0
exit 1
```

## Status codes

- `exit` is the same as `exit 0`.
- `exit 0` means success.
- `exit 1` means general failure.
- Valid codes are integers from `0` to `255`.

Codes may be integer literals, numeric expressions, variables, or constants:

```ly
$code = 0
exit $code

$$BASE = 1
exit $$BASE + 1
```

## Scope

`exit` exits the whole script, not just the current block, loop, or function.

It works at top level and inside:

- `if` / `elif` / `else`
- `repeat`
- `while`
- functions
- included files
- imported/exported functions when they run

## Errors

These are errors and do not pretend to exit successfully:

- strings
- booleans
- lists
- decimal values
- negative values
- values greater than `255`
- malformed trailing text

Diagnostics are line-numbered.
