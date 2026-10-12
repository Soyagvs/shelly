# Comments

Shelly comments must start and end with `#` markers outside strings.

## Line comments

```ly
# explain the next command #
shellout "hello"
```

A one-line comment opens and closes on the same line.

## Block comments

```ly
#
This whole block is ignored.
Use it for notes that span lines.
#
shellout "after the comment"
```

## Rules

- A comment that reaches end of file without a closing `#` is an error.
- The error includes the opening line number.
- `#` inside strings is literal text, not a comment marker.

```ly
shellout "# not a comment #"
```
