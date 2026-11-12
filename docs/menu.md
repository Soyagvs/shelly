# Interactive menu blocks

Use a `menu` block to select a value. TTYs get keyboard navigation; pipes and other non-TTY input use a deterministic numbered prompt.

## Syntax

```shelly
menu "Security dashboard" -> $section
subtitle "Choose what you want to inspect"
theme panel
default 1
option "Summary" -> "summary"
hint "Current posture and findings"
option "Ports and services" -> "ports"
hint "Network exposure"
end
```

The exact grammar is:

```text
menu "Title" -> $destination
subtitle "..."
theme panel|minimal
default N
option LABEL_EXPR -> VALUE_EXPR
hint "..."
end
```

- `menu` requires a scalar title and a mutable variable destination.
- `subtitle`, `theme`, and `default` are optional; the default theme is `panel` and the default selection is `1`.
- `LABEL_EXPR` is evaluated as a string expression. `VALUE_EXPR` may produce any Shelly value, which is stored unchanged.
- `hint` attaches to the immediately preceding option; an option has at most one hint.
- Visible labels must be unique, and a menu must contain at least one option. `default N` is one-based and must be within the option range.
- Arbitrary nested blocks are unsupported. A menu is a line-oriented block; use `end` to close it.

## TTY controls and display

When stdin and stdout are TTYs, use Up/Down arrows or `j`/`k` to move; navigation wraps. Press Enter to select, Escape to cancel, and resize to redraw. `panel` provides the bordered presentation; `minimal` omits the panel framing. Long text wraps with safe continuation indentation; narrow terminals preserve markers and bounds, splitting unbreakable text when necessary and using a visible ellipsis when a glyph cannot fit.

The menu does not use an alternate screen, mouse input, typeahead, or custom Ctrl-C handling. Ctrl-C keeps normal process interruption behavior. With no color, selection remains readable through text markers; ANSI control sequences are used only for cursor/redraw behavior.

## Pipes, errors, and dry-run

In pipe mode the title, subtitle, numbered options, and hints are printed as plain text. Hints are secondary lines attached to their option. The first input line selects by a one-based number or an exact visible label. Other input is invalid; EOF fails with `menu input closed`. Escape cancellation in a TTY fails with `menu cancelled`.

`choose` is removed. Scripts using it receive a migration diagnostic directing them to a `menu` block; there is no compatibility alias. Other structural diagnostics cover empty menus, invalid defaults, duplicate labels, unsupported nested blocks, misplaced or repeated hints, and missing `end`.

`--dry-run` does not prompt or read stdin. Menu selection has no result in dry-run, so it reports `dry-run unavailable-result: menu requires interactive input` and remains fail-closed. Normal execution and dry-run retain the existing audit semantics: menu input and outcomes are recorded according to the audit stream, while dry-run records the blocked/planned action rather than a selected value.

## Related guides

- [Commands and input](commands.md) — command reference.
- [Conditionals](conditionals.md) — branch on the selected value.
- [Error handling](errors.md) — recover from runtime failures.
- [Dry-run and audit logs](dry-run.md) — audit behavior.
