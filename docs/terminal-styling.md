# Style terminal values safely

Use `style(...)` to add a constrained terminal style to a value passed to `shellout`. It produces a value; it is not a general terminal-control protocol, and Markdown presentation never emits ANSI escapes.

## 🚀 Quick path

```shelly
$name = "Ada"
$label = style("bold,bright_cyan", "Hello $name")
shellout $label
shellout style("white on_blue", "status: ready")
```

The form is `style(STYLE, VALUE)`. Both arguments are ordinary Shelly expressions. `STYLE` must evaluate to a non-empty descriptor; `VALUE` is converted to a scalar. Styled values can be assigned, interpolated, nested, or passed to `shellout`; they are also supported in menu titles, subtitles, option labels, and hints.

## Executable control flow

`style(...)` and ordinary `shellout ""` are executable statements, so they can appear inside control-flow blocks, functions, and nested combinations. This complete example is runnable and covers conditionals, all loop forms, functions, error handling, retries, and nesting:

```shelly
if true
    shellout style("red", "if")
    shellout ""
end
case "branch"
    when "branch"
        shellout style("green", "case")
        shellout ""
end
for $item in ["for"]
    shellout style("blue", $item)
    shellout ""
end
$i = 0
while $i < 1
    $i = $i + 1
    shellout style("yellow", "while")
    shellout ""
end
repeat 1
    shellout style("magenta", "repeat")
    shellout ""
end
fn report() {
    shellout style("cyan", "function")
    shellout ""
}
call report()
try
    shellout style("white", "try")
    shellout ""
catch
    shellout style("bright_red", "catch")
    shellout ""
end
retry 2
    shellout style("red", "retry")
    shellout ""
end
if true
    repeat 1
        fn nested() {
            shellout style("bold", "nested")
            shellout ""
        }
        call nested()
    end
end
```

## Safe vocabulary

Descriptors contain one or more names separated by commas and/or whitespace:

| Role | Names |
| --- | --- |
| Foreground | `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white` |
| Bright foreground | `bright_black` through `bright_white` using the same color names |
| Background | `on_black` through `on_white` |
| Bright background | `on_bright_black` through `on_bright_white` |
| Attributes | `bold`, `dim`, `italic`, `underline`, `blink`, `reverse`, `strikethrough`, `reset` |

For example, `"bold red on_white"` is valid. Shelly emits a reset after the value. Raw ANSI sequences, numeric codes, unknown names, empty descriptors, and empty tokens are rejected. `style` validates the descriptor but does not sanitize terminal control bytes already present in `VALUE`.

## Palette roles

`lib/palette.shelly` exports semantic descriptors as ordinary constants, not additional native style names:

```shelly
import "lib/palette.shelly"
shellout style($SUCCESS, "completed")
shellout style($WARNING, "check configuration")
shellout style($ERROR, "failed")
```

| Constant | Role | Descriptor |
| --- | --- | --- |
| `$SUCCESS` | completed state | `bright_green` |
| `$WARNING` | caution | `bright_yellow` |
| `$ERROR` | failure | `bright_red` |
| `$INFO` | information | `bright_cyan` |
| `$MUTED` | secondary content | `dim` |
| `$CRITICAL` | urgent attention | `bold bright_white on_red` |
| `$TITLE` | section title | `bold bright_cyan` |

See the palette module for the remaining generic aliases. Resolve the import path relative to the importing script.

## Color, NO_COLOR, and output

```text
shelly --color auto SCRIPT
shelly --color always SCRIPT
shelly --color never SCRIPT
```

`auto` is the default and emits ANSI only when stdout is a TTY. `always` emits ANSI when redirected; `never` emits plain text. `NO_COLOR` takes precedence over every policy, including `always`, and disables ANSI. Only `auto`, `always`, and `never` are accepted.

Color policy controls terminal emission, not descriptor validation. Prefer `never` for consumers requiring plain text and `always` only for consumers that explicitly support ANSI. These runtime rules are separate from portable Markdown styling: documentation colors or headings do not change Shelly output.

## Check, dry-run, and redaction

`check` validates reachable modules without executing scripts, so it produces no styled script output. `--dry-run` skips effectful actions and produces no `shellout` output. With `--audit-log`, simulated actions are recorded; styling itself is not an effect to execute or audit.

Normal redaction still applies. Values from the secret facility remain redacted in stdout, stderr, audit logs, and errors; adding ANSI styling cannot reveal them.

## Errors and non-goals

Common errors include `style requires exactly two arguments`, `style requires a nonempty style descriptor`, `malformed style token`, and `unknown style token: NAME`. Appearance varies with terminal theme, font, platform, and accessibility settings.

v1 does not support arbitrary ANSI injection, RGB/256-color/truecolor syntax, hyperlinks, cursor control, terminal capability detection beyond the stdout TTY check, native semantic aliases, a `print` command, or editor grammar changes.

## Related guides

- [Commands](commands.md) — CLI options and output behavior.
- [Dry run and audit](dry-run.md) — simulation boundaries.
- [Secrets](secrets.md) — redaction behavior.
