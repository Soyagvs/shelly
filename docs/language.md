# Shelly language guide

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Shelly scripts use the `.shelly` extension. One instruction per line. No semicolons.

## Mental model

```shelly
$name = "Ada"          # mutable variable #
$$APP = "Shelly"       # immutable constant #
ask "Name? " -> $name  # result binding #
```

- `$name` stores a mutable value.
- `$$NAME` stores an immutable constant.
- `->` means “bind the result into this variable”.
- Commas are for normal argument lists.
- Blocks close with `end`.

## Reference

| Topic | Guide |
| --- | --- |
| Comments | [comments.md](comments.md) |
| Variables | [variables.md](variables.md) |
| Constants | [constants.md](constants.md) |
| Strings and interpolation | [strings.md](strings.md) |
| Numeric expressions | [numeric-expressions.md](numeric-expressions.md) |
| Lists | [lists.md](lists.md) |
| Commands and input | [commands.md](commands.md) |
| Exit | [exit.md](exit.md) |
| Functions | [functions.md](functions.md) |
| Conditionals | [conditionals.md](conditionals.md) |
| Repeat loops | [repeat.md](repeat.md) |
| While loops | [while.md](while.md) |
| Modules | [modules.md](modules.md) |

## Run

```sh
cargo run -- path/to/script.shelly
```

Complete showcase: [`../example-01.shelly`](../example-01.shelly).

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Not implemented yet

- Function return values.
- Logical operators `and`, `or`, and `not`.
- Parenthesized condition expressions.
- Nested lists.
- Ordered string filters.
- Multi-index, range, or filter mutation targets.
- Exporting normal variables.
