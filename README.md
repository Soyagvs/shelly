# Shelly

Shelly is a Bash 2.0 style shell and script runner for `.ly` scripts. It is designed to make everyday scripting readable: variables, commands, lists, blocks, functions, modules, and explicit exits in one small language.

## Status

This repository contains the current Shelly interpreter prototype written in Rust.

## Run a script

During development:

```sh
cargo run -- path/to/script.ly
```

After installing the binary, the intended command is:

```sh
shelly path/to/script.ly
```

Running without a script prints a short usage hint and exits successfully.

## Quick example

```ly
$name = "Agustin"
$$APP = "Shelly"
$names = ["Ana", "Luis", "Mora"]

shellout "Hello $name from $$APP"
shellout "First name: $names[first]"

ask "Continue? " -> $answer
if $answer != "yes"
    shellout "Cancelled"
    exit 1
end

command "pwd" -> $cwd
shellout "Running from $cwd"
exit 0
```

See [`example-01.ly`](example-01.ly) for a complete runnable showcase.

## Language guide

Start with the overview: [`docs/language.md`](docs/language.md).

Focused references:

- [Comments](docs/comments.md)
- [Variables](docs/variables.md)
- [Constants](docs/constants.md)
- [Strings and interpolation](docs/strings.md)
- [Numeric expressions](docs/numeric-expressions.md)
- [Lists](docs/lists.md)
- [Commands and input](docs/commands.md)
- [Exit](docs/exit.md)
- [Functions](docs/functions.md)
- [Conditionals](docs/conditionals.md)
- [Repeat loops](docs/repeat.md)
- [While loops](docs/while.md)
- [Modules: import, include, export](docs/modules.md)

## Core mental model

- `$name` is a mutable variable.
- `$$NAME` is an immutable constant.
- `->` means a command or block binds a result into a variable, for example `ask "Name? " -> $name`, `command "pwd" -> $cwd`, and `repeat 3 -> $i`.
- Commas separate ordinary arguments, for example function calls and list mutation commands.
- Blocks use `end`.
- Indexing is 1-based.
- `exit 0` means success; `exit 1` means general failure.

## Development checks

```sh
cargo fmt --check
cargo test
printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly
```
