# Ask for a choice with an interactive menu

Use `choose` to select an option and store its value. TTYs get a keyboard menu; pipes and other non-TTY input keep the numbered, line-oriented protocol.

## 🚀 Quick path

```shelly
choose "Deploy target?" ["staging", "production"] -> $target
shellout "target=$target"
```

Options are shown with one-based numbers. The destination receives the selected value, not its number.

## 📖 Syntax and rules

```shelly
choose "Question" [choice("Staging", "staging"), choice("Production", "prod")] default 2 -> $target
```

- Options may be a list literal or list expression and must evaluate to a non-empty list.
- `choice(LABEL, VALUE)` displays `LABEL` and stores `VALUE`. `LABEL` must be non-empty; values may be nested.
- Without `choice`, an option's formatted value is both its visible label and selected value.
- `default N` is optional and one-based. It highlights option `N` in a TTY menu; `N` must be at least 1.
- The destination must be a mutable variable.

## 🖥️ TTY behavior

When stdin and stdout are TTYs, `choose` shows a keyboard menu:

- Use Up/Down arrows to move; navigation wraps.
- Enter confirms the highlighted option. Escape cancels with `choose cancelled`.
- The default option starts highlighted; without a default, the first option is highlighted.
- A terminal resize redraws the menu.

The menu does not use an alternate screen, mouse input, typeahead, or custom Ctrl-C handling. Ctrl-C retains normal process interruption semantics. Rendering remains readable without color: the selection is marked with text, and terminal control sequences are used only for cursor/menu redraw. `NO_COLOR` expectations for Shelly styling remain in force; this menu does not promise additional ANSI color styling.

## 📟 Pipes and non-TTY input

Non-TTY input prints the numbered menu and accepts either a one-based number or an exact visible label. Empty or invalid lines print the prompt again and retry. EOF stops retrying and fails with `choose input closed`.

`--dry-run` reports `dry-run unavailable-result: choose requires interactive input` without prompting or reading stdin.

## ⚠️ Errors

An empty options list, invalid `default N`, or non-mutable destination is rejected. TTY Escape cancellation and non-TTY EOF are reported as errors.

## Related guides

- [Conditionals](conditionals.md) — branch on the selected value.
- [Lists](lists.md) — build and access option lists.
- [Error handling](errors.md) — recover from runtime failures.
