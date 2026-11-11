# Terminal styling

Terminal styling v1 adds a safe, value-producing `style(...)` expression. It is intended for values passed to `shellout`, not for arbitrary terminal control.

## Syntax

```shelly
style(STYLE, VALUE)
```

`STYLE` and `VALUE` are ordinary Shelly expressions. The first must evaluate to a non-empty style descriptor; the second is converted to a scalar value. A descriptor is one or more vocabulary names separated by commas and/or whitespace. The result is a styled scalar that can be assigned, interpolated, nested, or passed directly to `shellout`.

```shelly
$name = "Ada"
$label = style("bold,bright_cyan", "Hello $name")
shellout $label
shellout style("white on_blue", "status: ready")
$nested = style("underline", style("red", "warning"))
shellout $nested
```

The expression uses existing expression/call syntax. No external editor grammar change is required for `style(...)`.

## Safe style vocabulary

The descriptor accepts only these finite names:

- Foreground: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`.
- Bright foreground: `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`.
- Background: `on_black`, `on_red`, `on_green`, `on_yellow`, `on_blue`, `on_magenta`, `on_cyan`, `on_white`.
- Bright background: `on_bright_black`, `on_bright_red`, `on_bright_green`, `on_bright_yellow`, `on_bright_blue`, `on_bright_magenta`, `on_bright_cyan`, `on_bright_white`.
- Attributes: `bold`, `dim`, `italic`, `underline`, `blink`, `reverse`, `strikethrough`, `reset`.

Names can be combined, for example `"bold red on_white"`. Shelly emits a reset after the value. Raw ANSI escape sequences, numeric codes, unknown names, empty descriptors, and empty tokens are rejected in the `STYLE` descriptor. `VALUE` is ordinary data and may already contain terminal control bytes; `style` validates the descriptor but does not sanitize general output.

## Palette library

Semantic names are ordinary exported constants in the importable library `lib/palette.shelly`; they are not native style names.

```shelly
import "lib/palette.shelly"

shellout style($SUCCESS, "completed")
shellout style($WARNING, "check configuration")
shellout style($ERROR, "failed")
shellout style($INFO, "details")
shellout style($NOTICE, "notice")
shellout style($MUTED, "less important")
shellout style($HIGHLIGHT, "important")
```

The supplied aliases are `$SUCCESS = "bright_green"`, `$WARNING = "bright_yellow"`, `$ERROR = "bright_red"`, `$INFO = "bright_cyan"`, `$NOTICE = "bright_blue"`, `$MUTED = "dim"`, and `$HIGHLIGHT = "bold"`. Import paths are resolved relative to the importing script, so use the path that reaches your copy of `lib/palette.shelly`.

## Color emission policy

Use the CLI option:

```text
shelly --color auto SCRIPT
shelly --color always SCRIPT
shelly --color never SCRIPT
```

`auto` is the default and emits ANSI only when stdout is a TTY. `always` emits ANSI even when stdout is redirected. `never` strips styling. `NO_COLOR` takes precedence over every policy, including `--color always`; when it is present, styled output is emitted without ANSI. The option accepts only `auto`, `always`, or `never`; any other value is an error.

Consequently, redirecting `auto` output produces plain text, while redirecting `always` output preserves ANSI unless `NO_COLOR` is set:

```text
shelly --color auto report.shelly > report.txt
shelly --color always report.shelly > report.ansi.txt
```

Color policy affects terminal emission, not the value-level validation performed by `style`.

## Check, dry-run, and audit

`check` validates scripts and their reachable modules without executing them, so it produces no styled script output. `--dry-run` does not execute effectful actions and produces no `shellout` output, including styled output. With `--audit-log`, dry-run records the simulated actions in the audit file; styling itself is not an effect to execute or audit.

```text
shelly check report.shelly
shelly --dry-run --audit-log report.jsonl report.shelly
```

The audit log and diagnostics use Shelly's normal redaction behavior. Values obtained through the secret facility remain redacted when styled; adding ANSI styling cannot reveal the underlying secret in stdout, stderr, or audit/error output.

## Portability and errors

ANSI support depends on the terminal or consumer. The vocabulary maps to conventional ANSI SGR colors and attributes, but exact appearance varies by terminal theme, font, platform, and accessibility settings. `auto` relies on whether stdout is a TTY; pipelines, files, CI logs, and other redirected destinations are therefore plain by default. Prefer `never` when a downstream consumer requires plain text and `always` only when the consumer explicitly supports ANSI.

Typical errors include:

- `style requires exactly two arguments` for a missing or extra argument;
- `style requires a nonempty style descriptor` for an empty descriptor;
- `malformed style token` for an empty token; and
- `unknown style token: NAME` for a name outside the vocabulary.

## Explicit non-goals

v1 does not add a `print` command, native semantic palette aliases, arbitrary ANSI escape injection, RGB/256-color/truecolor syntax, hyperlinks or cursor control, terminal capability detection beyond the stdout TTY check, or external editor grammar changes. Styling is not a general-purpose terminal protocol; use normal Shelly commands and platform-specific tools for behavior outside this vocabulary.
