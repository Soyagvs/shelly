# Strings and interpolation

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Strings use double quotes and can include variables, constants, and list selectors.

```shelly
$name = "Ada"
$$APP = "Shelly"

shellout "hello $name from $$APP"
```

## Rules

- Strings use double quotes.
- Single quotes are ordinary text inside a string.
- Interpolation works for variables, constants, and list selectors.
- `#` inside a string is text, not a comment.
- Use escapes for special characters.

## Escapes

| Escape | Meaning |
| --- | --- |
| `\n` | newline |
| `\t` | tab |
| `\"` | double quote |
| `\\` | backslash |

## Interpolation examples

```shelly
$names = ["Ana", "Luis", "Mora"]

shellout "first: $names[1]"
shellout "selected: $names[1,3]"
shellout "# not a comment #"
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
