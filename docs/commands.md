# Commands and input

Shelly has built-in commands for output, input, prompts, and system shell execution.

```ly
shellout "hello"
shellout $name
shellin $answer
ask "Name? " -> $name
command "printf streamed"
command "printf captured" -> $value
```

## `shellout`

Prints the evaluated value and adds a newline.

```ly
shellout "hello"
shellout $name
shellout "hello $name"
```

## `shellin`

Reads one line from stdin and stores it without the line ending.

```ly
shellin $answer
```

The destination must be a variable.

## `ask`

Prints a prompt, flushes stdout, reads one line, trims the line ending, and stores it in a variable.

```ly
ask "Name? " -> $name
```

Rules:

- The prompt before `->` must be a string.
- The destination after `->` must be a variable.
- Constants and other values are rejected as destinations.
- The old comma form is rejected with migration guidance.
- `ask` is not print-only; use `shellout` when you only need to print.

## `command`

Streams a system shell command through `sh -c`.

```ly
command "printf hello"
```

A non-zero status fails the Shelly script.

## Command capture

Capture stdout with arrow binding:

```ly
command "pwd" -> $cwd
shellout "Current directory: $cwd"
```

Rules:

- Captured stdout has trailing newlines trimmed.
- Captured stdout is not printed automatically.
- The destination must be a variable, not a constant.
- The old standalone `capture` syntax is removed.
- Shell commands use operating-system shell quoting rules inside the command string.
