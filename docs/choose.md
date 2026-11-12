# Ask for a choice with an interactive menu

Use `choose` to print a numbered menu, read a selection from stdin, and store the selected option value.

## 🚀 Quick path

```shelly
choose "Deploy target?" ["staging", "production"] -> $target
shellout "target=$target"
```

Shelly prints a question, numbered options, and `Choose: `. The user may enter the number (`2`) or the exact displayed selection (`production`). The destination receives `production`, not `2`; menu text is stdout only.

## 📖 Syntax and rules

```shelly
choose "Question" ["A", "B", "C"] -> $choice
```

- Options may be a list literal or list expression and must evaluate to a non-empty list.
- Nested values are accepted and displayed using their formatted value; that exact displayed text can be entered.
- The destination must be a mutable variable.
- There is no default selection or count limit.
- Invalid input repeats the prompt until a valid number or exact displayed selection is entered.

```shelly
$name = "Ada"
choose "Hi $name, pick a color" ["red", "blue"] -> $color
shellout "You picked $color"
```

```shelly
$options = ["small", "medium", "large"]
choose "Size?" $options -> $size
```

## ⚠️ Interactive use and errors

`choose` reads stdin and is not suitable for noninteractive use. At EOF, empty input is retried indefinitely. An empty or invalid options value, or a non-mutable destination, is rejected.

## Related guides

- [Conditionals](conditionals.md) — branch on the selected value.
- [Lists](lists.md) — build and access option lists.
