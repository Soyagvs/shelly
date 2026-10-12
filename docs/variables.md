# Variables

Variables store mutable values and use `$name` syntax.

```shelly
$name = "world"
$empty
$count = 42
$enabled = true
shellout "hello $NAME"
```

## Rules

- Variables start with one `$`.
- Variable names are case-insensitive: `$name` and `$NAME` refer to the same variable.
- A bare variable declaration, such as `$empty`, creates an empty value.
- Identifiers must start with an ASCII letter, followed by ASCII letters, numbers, or `_`.
- Variables may store strings, numbers, booleans, lists, empty values, or captured command output.

## Assignment

```shelly
$name = "Ada"
$count = 1 + 2 * 3
$enabled = true
$names = ["Ana", "Luis"]
command "pwd" -> $cwd
```

`->` means a command result is bound into a variable. Use it for result binding, not for ordinary argument lists.
