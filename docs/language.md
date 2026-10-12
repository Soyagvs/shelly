# Shelly language guide

Shelly scripts use the `.ly` extension. The interpreter is line-oriented: one instruction per line, no semicolons.

## Guided mental model

Start with values, then commands, then blocks. `$name` stores a mutable variable, `$$NAME` stores an immutable constant, and most commands read values from ordinary comma-separated arguments.

The arrow `->` is special: it means the command or block produces or binds a result into a variable:

```ly
ask "Name? " -> $name
command "pwd" -> $cwd
repeat 3 -> $i
```

Use commas for normal argument lists. Use `->` only for result binding.

Run a script with:

```sh
cargo run -- path/to/script.ly
```

## Current language reference

- [Comments](comments.md) — explicit `# ... #` line and block comments.
- [Variables](variables.md) — mutable `$name` values and empty declarations.
- [Constants](constants.md) — immutable uppercase `$$NAME` values.
- [Strings and interpolation](strings.md) — double-quoted strings, escapes, `$name`, `$$NAME`, and list selectors inside strings.
- [Numeric expressions](numeric-expressions.md) — integers, floats, operators, precedence, parentheses, and numeric variables/constants.
- [Lists](lists.md) — list literals, 1-based selectors, ranges, filters, and mutations.
- [Commands and input](commands.md) — `shellout`, `shellin`, `ask`, and `command`/capture.
- [Exit](exit.md) — process-wide termination with status codes `0..255`.
- [Functions](functions.md) — `fn`/`end`, parameters, defaults, local scope, and `global`.
- [Conditionals](conditionals.md) — `if`/`elif`/`else`, comparisons, and truthiness.
- [Repeat loops](repeat.md) — counted loops and optional local 1-based index binding.
- [While loops](while.md) — condition-based loops with a safety limit.
- [Modules](modules.md) — `import`, `include`, and `export`.

## Complete showcase

See [`../example-01.ly`](../example-01.ly) for a broad runnable example covering the current language.

## Not implemented yet

These are intentionally not part of the current language yet:

- Function return values.
- Logical operators `and`, `or`, and `not`.
- Parenthesized condition expressions.
- Nested lists.
- Ordered string filters.
- Multi-index, range, or filter mutation targets.
- Exporting normal variables.
