# Choose menus

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

`choose` asks a question, prints a numbered menu, and stores the selected option value.

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

The user may type either the number (`2`) or the exact option text (`production`). The destination receives `production`, not `2`.

## Rules

- Syntax is `choose "Question" ["A", "B", "C"] -> $choice`.
- The question uses the same string literal and interpolation behavior as `ask`.
- Options must evaluate to a non-empty list of scalar values.
- The destination after `->` must be a mutable variable.
- Invalid input repeats the `Choose: ` prompt until the user enters a valid number or exact option text.

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
