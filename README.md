<p align="center">
  <img width="1672" height="941" alt="Shelly preview" src="https://github.com/user-attachments/assets/48130392-11ba-41b5-b655-71e32c88f1cd" />
</p>

<p align="center">
  <img alt="Releases" src="https://img.shields.io/github/v/release/Soyagvs/shelly?include_prereleases&style=for-the-badge&label=release&color=7c3aed" />
  <img alt="Downloads" src="https://img.shields.io/github/downloads/Soyagvs/shelly/total?style=for-the-badge&label=downloads&color=7c3aed" />
  <img alt="Stars" src="https://img.shields.io/github/stars/Soyagvs/shelly?style=for-the-badge&label=stars&color=7c3aed" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-7c3aed?style=for-the-badge" />
</p>

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=3&color=7c3aed" width="100%" alt="purple divider" />
</p>

# Shelly

Shelly is a Bash 2.0 style shell and script runner for `.shelly` scripts.

Small scripts. Clear syntax. One readable way to compose variables, commands, lists, blocks, functions, modules, and explicit exits.

## Latest release

**v3.0.1** is the latest public-ready Shelly release.

- Linux/macOS installer: `curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh`
- Release binaries are built for Linux x86_64, macOS Intel, macOS Apple Silicon, and Windows x86_64.
- Language highlights: lists, selectors, filters, `for`, `while`, `repeat`, explicit `call`, function `return`, `break`, `continue`, logical `and` / `or` / `not`, modules, and `command` for system tools.
- License: MIT.

## Install

Linux and macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh
```

Create a script named `hello.shelly`:

```shelly
shellout "Hello from Shelly"
```

Run it:

```sh
shelly hello.shelly
```

Windows users can download `shelly-windows-x86_64.zip` from the latest GitHub Release, unzip it, and run:

```powershell
.\shelly.exe hello.shelly
```

Manual downloads are also available on GitHub Releases:

- Linux: `shelly-linux-x86_64.tar.gz`
- macOS with Intel chip: `shelly-macos-x86_64.tar.gz`
- macOS with Apple silicon: `shelly-macos-arm64.tar.gz`
- Windows: `shelly-windows-x86_64.zip`

## Run from source

```sh
cargo run -- path/to/script.shelly
```

## Taste

```shelly
$name = "Agustin"
$$APP = "Shelly"
$names = ["Ana", "Luis", "Mora"]

shellout "Hello $name from $$APP"
shellout "First name: $names[first]"

ask "Continue? " -> $answer
if $answer != "yes"
    exit 1
end

command "pwd" -> $cwd
shellout "Running from $cwd"
exit 0
```

Full showcase: [`example-01.shelly`](example-01.shelly).

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Docs

Start here: [`docs/language.md`](docs/language.md).

| Topic | Guide |
| --- | --- |
| Comments | [`docs/comments.md`](docs/comments.md) |
| Variables | [`docs/variables.md`](docs/variables.md) |
| Constants | [`docs/constants.md`](docs/constants.md) |
| Strings | [`docs/strings.md`](docs/strings.md) |
| Numeric expressions | [`docs/numeric-expressions.md`](docs/numeric-expressions.md) |
| Lists | [`docs/lists.md`](docs/lists.md) |
| Commands and input | [`docs/commands.md`](docs/commands.md) |
| Exit | [`docs/exit.md`](docs/exit.md) |
| Functions | [`docs/functions.md`](docs/functions.md) |
| Conditionals | [`docs/conditionals.md`](docs/conditionals.md) |
| Repeat loops | [`docs/repeat.md`](docs/repeat.md) |
| While loops | [`docs/while.md`](docs/while.md) |
| For loops | [`docs/for.md`](docs/for.md) |
| Modules | [`docs/modules.md`](docs/modules.md) |

## Editor support

- Neovim: [`editors/nvim`](editors/nvim)
- Shelly artwork: [`shelly.svg`](shelly.svg)

## Mental model

- `$name` is mutable.
- `$$NAME` is immutable.
- `->` binds a result into a variable.
- Commas separate normal arguments.
- Blocks close with `end`.
- Indexing starts at `1`.
- `exit 0` succeeds; `exit 1` fails.

## Checks

```sh
cargo fmt --check
cargo test
printf 'yes\nblue\n' | cargo run --quiet -- example-01.shelly
```
