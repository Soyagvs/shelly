# Choose menus

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

`choose` is interactive: it asks a question, prints a numbered menu to stdout, and stores the selected option value.

```shelly
choose "Deploy target?" ["staging", "production"] -> $target
shellout "target=$target"
```

Shelly prints:

```text
Deploy target?
1) staging
2) production
Choose: 
```

The user may type either the number (`2`) or the exact displayed selection (`production`). The destination receives `production`, not `2`; the menu text is only stdout, while the selected value is stored.

## Rules

- Syntax is `choose "Question" ["A", "B", "C"] -> $choice`.
- The options may be a list literal or a list expression, and must evaluate to a non-empty list.
- Nested values are accepted; each option is displayed using its formatted value, and that exact displayed text can be entered.
- There is no default selection or count limit. The destination after `->` must be a mutable variable.
- Invalid input repeats the `Choose: ` prompt until the user enters a valid number or exact displayed selection.
- Because it reads stdin interactively, `choose` is not suitable for noninteractive use: at EOF, an empty input is retried indefinitely.

## Examples

```shelly
$name = "Ada"
choose "Hi $name, pick a color" ["red", "blue"] -> $color
shellout "You picked $color"
```

```shelly
$options = ["small", "medium", "large"]
choose "Size?" $options -> $size
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
