# Commands and input

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Shelly has built-in commands for output, input, prompts, and system shell execution.

```shelly
shellout "hello"
ask "Name? " -> $name
choose "Color?" ["red", "blue"] -> $color
command "pwd" -> $cwd
arg 1 -> $name
hasArg "--dry-run" -> $dryrun
args -> $all
```

## Rules

- `shellout` prints a value and adds a newline.
- `shellin` reads one stdin line into a variable.
- `ask` prints a prompt, reads one line, and stores it with `->`.
- `choose` prints a numbered menu, accepts a number or exact option text, and stores the selected value; see [choose.md](choose.md).
- `command` runs a system command.
- `arg N -> $var`, `hasArg VALUE -> $var`, and `args -> $var` read arguments passed after the script path.
- `log "identifier"` records structured action markers for a block without silencing normal output; see [logging.md](logging.md).
- Failed system commands are ordinary errors and can be handled with `try` / `catch`.
- `command "..."` streams output directly.
- `command "..." -> $var` captures stdout into a variable and still fails on non-zero exit.
- `command "..." -> $stdout, $stderr, $code` captures stdout, stderr, and the numeric exit code without failing on non-zero exit; when the operating system provides no code, Shelly stores `1`.
- `exists "path" -> $var` and `contains $value "thing" -> $var` capture predicate results as booleans (`true` or `false`).
- `create`, `copy`, `move`, `rename`, `remove`, `read`, `write`, `append`, `isFile`, `isDir`, `size`, `tempFile`, `tempDir`, and `files` cover common file automation; see [files.md](files.md).
- `command` is the bridge to OS tools such as `mv`, `cp`, `git`, or `curl`.
- Standard helpers such as `trim`, `join`, `toNumber`, `basename`, `now`, and `randomInt` transform values without shelling out; see [helpers.md](helpers.md).

## Output

```shelly
$name = "Ada"

shellout $name
shellout "hello $name"
```

Use `"..."` for one-line output and triple backticks for multiline output.

````shelly
shellout ```
line one
line two
```
````

## Input

```shelly
ask "Favorite color? " -> $color
choose "Favorite color?" ["red", "blue"] -> $color
shellin $line
```

## Script arguments

Arguments are the values after the script path in `shelly script.shelly ...`; they do not include `shelly` or the script path.

```shelly
arg 1 -> $name
arg 2 -> $mode
arg 99 -> $missing
hasArg "--dry-run" -> $dryrun
args -> $all

for $item in $all
    shellout "arg: $item"
end
```

`arg` is 1-based. Missing positions store `""`. `hasArg` checks exact equality against the evaluated value.

## Predicate capture

Predicate capture stores a boolean result in a variable.

```shelly
exists "README.md" -> $hasreadme
contains $name "gus" -> $hasgus
```

See [predicates.md](predicates.md) for full `exists`, `contains`, and logical composition details.

## Standard helpers

```shelly
trim "  Ada  " -> $name
split "a,b" "," -> $items
join $items ", " -> $text
randomInt 1 6 -> $roll
```

See [helpers.md](helpers.md) for all helper commands.

## Log blocks

```shelly
log "build"
    shellout "starting build"
    command "printf ok"
end
```

`log` writes structured markers to `logs/build-YYYY-MM-DD.log` while normal output still goes to the terminal. See [logging.md](logging.md) for storage rules and limitations.

## System commands

```shelly
command "printf hello"
command "pwd" -> $cwd
command "printf 'oops\n' >&2; exit 7" -> $out, $err, $code

shellout "current directory: $cwd"
shellout "command finished with code $code and stderr: $err"
```

`command "..."` streams stdout/stderr and fails the script on a non-zero exit. `command "..." -> $stdout` captures stdout, trims trailing newlines, and also fails on a non-zero exit. Use `command "..." -> $stdout, $stderr, $code` when the script should inspect failure itself; stdout and stderr are captured with trailing newlines trimmed, and `$code` is the process exit code or `1` when no code is available.

For portable file work, prefer Shelly path commands such as `write "file" $text`, `append "file" $text`, `read "file" -> $text`, `size "file" -> $bytes`, and `copy "from" "to"`; see [files.md](files.md). Use `command` when you need an OS-specific tool.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
