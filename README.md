<p align="center">
  <img width="2056" height="765" alt="Cuervo geométrico y logotipo Shelly сия" src="https://github.com/user-attachments/assets/975333ea-e6c7-421b-9795-089f07211ce6" />
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

**Shelly is a modern scripting language for readable automation with `.shelly` scripts.**

It is built for small, clear scripts that automate files, commands, structured data, menus, logs, helpers, modules, and safe checks.

## Latest release

**v3.2.0** is the latest public-ready Shelly release.

Installers:

```sh
curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh
```

```powershell
irm https://raw.githubusercontent.com/Soyagvs/shelly/main/install.ps1 | iex
```

Release binaries are available for Linux x86_64, macOS Intel, macOS Apple Silicon, and Windows x86_64.

## Quick start

Create `hello.shelly`:

````shelly
shellout "Hello from Shelly"

shellout ```
Hello
from multiple lines
```
````

Check it safely without executing commands:

```sh
shelly check hello.shelly
```

Run it:

```sh
shelly hello.shelly
```

## Why Shelly

| Need | Shelly gives you |
| --- | --- |
| Readable automation | One clear syntax for scripts, files, commands, and data. |
| Safe validation | `shelly check` validates scripts without executing them. |
| Structured scripting | Lists, dictionaries, nested values, and JSON helpers. |
| Practical workflows | File helpers, menus, logs, command status capture, and modules. |
| Editor support | VS Code diagnostics and Neovim syntax/icons. |

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

## Language highlights

- Variables, constants, strings, lists, dictionaries, nested values, and JSON helpers.
- `if`, `elif`, `else`, `case`, `repeat`, `while`, and iterable `for` loops.
- Explicit function calls with `call`, function `return`, `break`, and `continue`.
- Logical `and` / `or` / `not`, grouped conditions, `exists`, and `contains`.
- File commands/helpers, standard helpers, logs, menus, script arguments, and modules.
- Command stdout/stderr/status capture with `command "..." -> $out, $err, $code`.

## Install manually

Download a release asset from <https://github.com/Soyagvs/shelly/releases/latest>:

| Platform | Asset |
| --- | --- |
| Linux | `shelly-linux-x86_64.tar.gz` |
| macOS Intel | `shelly-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `shelly-macos-arm64.tar.gz` |
| Windows | `shelly-windows-x86_64.zip` |

## Run from source

```sh
cargo run -- path/to/script.shelly
```

## Script arguments

```sh
shelly script.shelly Ada --dry-run
```

```shelly
arg 1 -> $name
hasArg "--dry-run" -> $dryRun
args -> $all
```

## Mental model

- `$name` is mutable.
- `$$NAME` is immutable.
- `->` binds a result into a variable.
- Commas separate normal arguments.
- Blocks close with `end`.
- Indexing starts at `1`.
- `exit 0` succeeds; `exit 1` fails.

## Editor support

- VS Code: [`Soyagvs/shelly-vscode`](https://github.com/Soyagvs/shelly-vscode)
- Neovim: [`editors/nvim`](editors/nvim)
- File icons: [`docs/file-icons.md`](docs/file-icons.md)
- Artwork source: [`shelly.svg`](shelly.svg)

## Documentation

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
| File icons | [`docs/file-icons.md`](docs/file-icons.md) |
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
| Project structure | [`docs/project-structure.md`](docs/project-structure.md) |

## Development checks

```sh
cargo fmt --check
cargo test
```

## License

MIT. See [`LICENSE`](LICENSE).
