# Commands and input

## `shelly update`

`shelly update` is a manual-only command; it does not run automatically. It downloads the latest versioned GitHub Release asset for Linux x86_64, macOS x86_64/aarch64, or Windows x86_64, including both `shelly` and `shelly-lsp`, and verifies the archive's SHA-256 checksum sidecar. It requires permission to replace the installed binaries. GitHub network access and checksum validation cover the downloaded archive, not OS trust. On Windows replacement is handed to a helper after exit; restart Shelly after the handoff.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Shelly has built-in commands for output, input, prompts, collection mutation, and system shell execution.

Collection mutation uses `set PATH VALUE`, `append PATH VALUE`, `delete PATH`, and top-level scalar-list `prepend $list VALUE`. Command arguments are whitespace-separated; comma-separated command syntax is unsupported. Structural commas remain valid in literals and capture destinations. Paths may combine dot keys, bracket-string keys, and one-based list indexes; see [lists.md](lists.md) and [dictionaries.md](dictionaries.md). Nested `prepend`, `insert`, slices, and range replacement remain deferred. These commands are distinct from file `append` and `remove`.

```shelly
shellout "hello"
ask "Name? " -> $name
choose "Color?" ["red", "blue"] -> $color
command pwd -> $cwd
arg 1 -> $name
hasArg "--dry-run" -> $dryrun
args -> $all
```

## Rules

- `shellout` prints a value and adds a newline.
- `shellin` reads one stdin line into a variable.
- `ask` prints a prompt, reads one line, and stores it with `->`.
- `choose` prints a numbered menu, accepts a number or exact option text, and stores the selected value; see [choose.md](choose.md).
- `command <program> <args...>` runs a program directly, without shell parsing. Bare words, quoted argument atoms, and variables each become argv arguments. Add `timeout SECONDS` to set a per-process deadline; see [Timeouts](timeouts.md).
- `shell "..."` explicitly runs text through the system shell; use it for pipelines and redirections. Shell syntax is platform dependent.
- `arg N -> $var`, `hasArg VALUE -> $var`, and `args -> $var` read arguments passed after the script path.
- `log "identifier"` records structured action markers for a block without silencing normal output; see [logging.md](logging.md).
- Failed system commands are ordinary structured errors and can be handled with `try` / `catch`; inspect the error dictionary's `kind`, `message`, and process fields there.
- `command <program> <args...>` streams output directly.
- `command <program> <args...> -> $var` captures stdout into a variable and still fails on non-zero exit.
- `command <program> <args...> -> $stdout, $stderr, $code` captures stdout, stderr, and the numeric exit code without failing on non-zero exit; when the operating system provides no code, Shelly stores `1`.
- Output captures may be one variable or three variables; `shell` uses the same capture forms.
- The old `command "..."` syntax was removed; use `shell "..."` when you need shell text.
- `exists "path" -> $var` and `contains $value "thing" -> $var` capture predicate results as booleans (`true` or `false`).
- `create`, `copy`, `move`, `rename`, `remove`, `read`, `write`, `append`, `isFile`, `isDir`, `size`, `tempFile`, `tempDir`, and `files` cover common file automation; see [files.md](files.md).
- `command` is the bridge to OS tools such as `mv`, `cp`, `git`, or `curl`.

## External process waiting

`watch <program> <args...> -> $line` starts one external process and runs its block once for each stdout line, in order:

```shelly
watch printf "one\\ntwo\\n" -> $line
    shellout $line
end
```

The child is waited for after stdout reaches EOF, and the block finishes when the child exits. Stderr is not converted into events. A read, handler, spawn, or non-zero-exit error stops the watcher and cleans up the child; the error can be handled with `try` / `catch`. The `$line` binding is restored after the block. `wait` with no arguments blocks indefinitely until normal process interruption (Ctrl+C); it does not create a child or scheduler.
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
    command printf ok
end
```

`log` writes structured markers to `logs/build-YYYY-MM-DD.log` while normal output still goes to the terminal. See [logging.md](logging.md) for storage rules and limitations.

## System commands

```shelly
command printf hello
command pwd -> $cwd
command printf "hello world" -> $out
command printf "%s\\n" $cwd -> $out, $err, $code
shell "printf 'oops\\n' >&2; exit 7" -> $out, $err, $code

shellout "current directory: $cwd"
shellout "command finished with code $code and stderr: $err"
```

`command <program> <args...>` executes directly, so arguments are not interpreted as shell syntax. Quoted atoms remain one argument and variables are evaluated as arguments. Add `timeout SECONDS` after the arguments to bound that process; see [Timeouts](timeouts.md). It streams stdout/stderr and fails the script on a non-zero exit; catch that failure as a structured error with `kind`, `message`, and process fields (`program`, `args`, `code`, `stdout`, and `stderr`). A one-variable capture trims trailing newlines and also fails on a non-zero exit. Use three captures when the script should inspect failure itself; stdout and stderr are trimmed, and `$code` is the process exit code or `1` when no code is available. Use `shell "..."` for pipelines or redirections; it is shell/platform dependent.

For portable file work, prefer Shelly path commands such as `write "file" $text`, `append "file" $text`, `read "file" -> $text`, `size "file" -> $bytes`, and `copy "from" "to"`; see [files.md](files.md). Use `command` when you need an OS-specific tool.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
