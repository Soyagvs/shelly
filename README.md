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

Small scripts. Clear syntax. One readable way to compose variables, commands, lists, dictionaries, blocks, functions, modules, and explicit exits.

## Latest release

**v3.1.0** is the latest public-ready Shelly release.

- Linux/macOS installer: `curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh`
- Windows installer: `irm https://raw.githubusercontent.com/Soyagvs/shelly/main/install.ps1 | iex`
- Release binaries are built for Linux x86_64, macOS Intel, macOS Apple Silicon, and Windows x86_64.
- Language highlights: script arguments, nested lists, dictionaries, JSON helpers, `case`, file commands/helpers, standard helpers, `for`, `while`, `repeat`, explicit `call`, function `return`, `break`, `continue`, logical `and` / `or` / `not` with grouped conditions, `exists` / `contains` predicates, modules, and `command` with stdout/stderr/status capture.
- Editor support: `shelly check` validates scripts safely without executing them, enabling VS Code diagnostics.
- License: MIT.

## Install

Linux and macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh
```

Create a script named `hello.shelly`:

````shelly
shellout "Hello from Shelly"

shellout ```
Hello
from multiple lines
```
````

Check it without executing commands:

```sh
shelly check hello.shelly
```

Run it:

```sh
shelly hello.shelly
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/Soyagvs/shelly/main/install.ps1 | iex
```

Then run:

```powershell
shelly hello.shelly
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

choose "Continue?" ["yes", "no"] -> $answer
if $answer != "yes" or not contains $names "Luis"
    exit 1
end

if exists "README.md"
    shellout "README found"
end

case $answer
    when "yes"
        shellout "continuing"
    else
        shellout "stopping"
end

command "pwd" -> $cwd
command "printf 'warning\n' >&2; exit 7" -> $out, $err, $code
shellout "Running from $cwd"
shellout "last command code: $code stderr: $err"
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
| Language overview | [`docs/language.md`](docs/language.md) |
| Comments | [`docs/comments.md`](docs/comments.md) |
| Variables | [`docs/variables.md`](docs/variables.md) |
| Constants | [`docs/constants.md`](docs/constants.md) |
| Strings | [`docs/strings.md`](docs/strings.md) |
| Numeric expressions | [`docs/numeric-expressions.md`](docs/numeric-expressions.md) |
| Lists | [`docs/lists.md`](docs/lists.md) |
| Dictionaries | [`docs/dictionaries.md`](docs/dictionaries.md) |
| JSON helpers | [`docs/json.md`](docs/json.md) |
| Standard helpers | [`docs/helpers.md`](docs/helpers.md) |
| Logging | [`docs/logging.md`](docs/logging.md) |
| Files | [`docs/files.md`](docs/files.md) |
| Commands and input | [`docs/commands.md`](docs/commands.md) |
| Choose menus | [`docs/choose.md`](docs/choose.md) |
| Exit | [`docs/exit.md`](docs/exit.md) |
| Error handling | [`docs/errors.md`](docs/errors.md) |
| Functions | [`docs/functions.md`](docs/functions.md) |
| Conditionals | [`docs/conditionals.md`](docs/conditionals.md) |
| Predicates | [`docs/predicates.md`](docs/predicates.md) |
| Repeat loops | [`docs/repeat.md`](docs/repeat.md) |
| While loops | [`docs/while.md`](docs/while.md) |
| For loops | [`docs/for.md`](docs/for.md) |
| Modules | [`docs/modules.md`](docs/modules.md) |

## Editor support

- Neovim: [`editors/nvim`](editors/nvim)
- VS Code: [`Soyagvs/shelly-vscode`](https://github.com/Soyagvs/shelly-vscode)
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


### Script arguments

Pass arguments after the script path and read them in Shelly:

```sh
shelly script.shelly Ada --dry-run
```

```shelly
arg 1 -> $name
hasArg "--dry-run" -> $dryRun
args -> $all
```
