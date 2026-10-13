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
```

## Rules

- `shellout` prints a value and adds a newline.
- `shellin` reads one stdin line into a variable.
- `ask` prints a prompt, reads one line, and stores it with `->`.
- `choose` prints a numbered menu, accepts a number or exact option text, and stores the selected value; see [choose.md](choose.md).
- `command` runs a system command.
- `log "identifier"` records structured action markers for a block without silencing normal output; see [logging.md](logging.md).
- Failed system commands are ordinary errors and can be handled with `try` / `catch`.
- `command "..."` streams output directly.
- `command "..." -> $var` captures stdout into a variable.
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

## Predicate capture

Predicate capture stores a boolean result in a variable.

```shelly
exists "README.md" -> $hasReadme
contains $name "gus" -> $hasGus
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

shellout "current directory: $cwd"
```

For portable file work, prefer Shelly path commands such as `write "file" $text`, `append "file" $text`, `read "file" -> $text`, `size "file" -> $bytes`, and `copy "from" "to"`; see [files.md](files.md). Use `command` when you need an OS-specific tool.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
