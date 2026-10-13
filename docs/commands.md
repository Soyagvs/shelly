# Commands and input

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Shelly has built-in commands for output, input, prompts, and system shell execution.

```shelly
shellout "hello"
ask "Name? " -> $name
command "pwd" -> $cwd
```

## Rules

- `shellout` prints a value and adds a newline.
- `shellin` reads one stdin line into a variable.
- `ask` prints a prompt, reads one line, and stores it with `->`.
- `command` runs a system command.
- `command "..."` streams output directly.
- `command "..." -> $var` captures stdout into a variable.
- `command` is the bridge to OS tools such as `mv`, `cp`, `git`, or `curl`.

## Output

```shelly
$name = "Ada"

shellout $name
shellout "hello $name"
```

## Input

```shelly
ask "Favorite color? " -> $color
shellin $line
```

## System commands

```shelly
command "printf hello"
command "pwd" -> $cwd

shellout "current directory: $cwd"
```

Shelly does not need built-ins for every file operation. Use the operating system command that fits your environment.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
