# Strings and interpolation

Strings use double quotes.

```shelly
$name = "Ada"
$$APP = "Shelly"
shellout "hello $name from $$APP"
```

## Escapes

Supported escapes include:

- `\n` newline
- `\t` tab
- `\"` double quote
- `\\` backslash

Single quotes are ordinary text inside double-quoted strings.

## Interpolation

Strings can interpolate variables, constants, and list selectors:

```shelly
$name = "Ada"
$names = ["Ana", "Luis", "Mora"]
$$APP = "Shelly"

shellout "hello $name"
shellout "app: $$APP"
shellout "second: $names[2]"
shellout "selected: $names[1,3]"
```

A `#` inside a string is literal text, not a comment marker.

```shelly
shellout "# not a comment #"
```
