# Terminal tables

`table` prints a small, fixed-width terminal table. A table has one or more
`column` declarations, followed by zero or more `row` declarations and an
optional `footer`, then `end`:

```shelly
table border rounded
    column "Name"
    column "Score" align right
    row "Ada" 42
    row "Lin" 37
    footer "Total" 79
end
```

`border` is optional; the default is `rounded`. The supported modes are:

- `ascii`: `+`, `-`, and `|`
- `unicode`: square Unicode corners and junctions
- `rounded`: rounded Unicode outer corners

## Cells and columns

Each cell is one ordinary Shelly expression. Strings, numbers, variables,
constants, and `style(STYLE, VALUE)` expressions are supported:

```shelly
$name = "Ada"
$TITLE = "Person"
table border unicode
    column style("bold bright_cyan", $TITLE) align center
    column "Name"
    row 1 $name
end
```

Cells are separated by whitespace. Quoted strings and parenthesized
expressions may contain spaces. `column EXPR` defines a header; its optional
alignment is `align left`, `align center`, or `align right`. Data rows are
left-aligned. Footer cells use the corresponding column alignment. Text is
padded using visible terminal width, so ANSI styling does not distort layout.

Every row and the footer, when present, must contain exactly as many cells as
there are columns. A table must contain at least one column, and it supports
at most one footer. Width or shape mismatches are errors rather than silently
truncated output.

Table colors follow Shelly's normal [terminal styling](terminal-styling.md)
policy: `--color auto`, `--color always`, and `--color never` apply, and
`NO_COLOR` disables ANSI output even when `always` is requested. Styling is
still validated when color output is disabled.

Tables intentionally do not support `colspan`, merged cells, multiline cells,
or arbitrary per-cell options. Alignment is a column option only; border mode
is a table option only.
