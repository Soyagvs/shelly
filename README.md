
<div align="center">

<img src="https://github.com/user-attachments/assets/975333ea-e6c7-421b-9795-089f07211ce6" alt="Shelly — raven logo" width="100%" />

### A modern scripting language for readable automation.

**Write clear scripts. Automate real work. Go beyond Bash.**

[![Release](https://img.shields.io/github/v/release/Soyagvs/shelly?include_prereleases&style=flat-square&label=release&color=7c3aed)](https://github.com/Soyagvs/shelly/releases)
[![Downloads](https://img.shields.io/github/downloads/Soyagvs/shelly/total?style=flat-square&label=downloads&color=8b5cf6)](https://github.com/Soyagvs/shelly/releases)
[![Stars](https://img.shields.io/github/stars/Soyagvs/shelly?style=flat-square&color=a78bfa)](https://github.com/Soyagvs/shelly/stargazers)
[![GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-6d28d9?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/built_with-Rust-242038?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

[**Get started**](#-get-started) · [**Explore the language**](#-the-language) · [**Editor support**](#-editor-support) · [**Documentation**](#-documentation) · [**Releases**](https://github.com/Soyagvs/shelly/releases/latest)

</div>

---

## ✦ Why Shelly?

Shell scripts are useful. They can also become difficult to read, validate, and maintain as they grow.

**Shelly** brings a consistent syntax to everyday automation: files, system commands, structured data, interactive menus, and reusable modules. Scripts use the **`.shelly`** extension, and the interpreter is built in **Rust**.

<table>
<tr>
<td width="50%" valign="top">

**◈ Readable by design**

Variables, conditionals, loops, and functions in a clear, explicit syntax.

</td>
<td width="50%" valign="top">

**◇ Optional validation**

Use <code>shelly check</code> when you want to validate a script without executing its commands; running a script directly does not require this step.

</td>
</tr>
<tr>
<td valign="top">

**⌘ Built for automation**

Run system commands, capture output and status, and work with files.

</td>
<td valign="top">

**▣ More than plain text**

Lists, dictionaries, nested values, JSON helpers, and modules.

</td>
</tr>
</table>

## ⚡ Get started

**Latest release featured in this README: `v3.3.1` · [Download release assets](https://github.com/Soyagvs/shelly/releases/latest)

<details open>
<summary><strong>Linux / macOS</strong></summary>

```sh
curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh
```

When needed, the installer adds its directory to your `PATH` in the supported shell profile (`~/.bashrc` or `~/.zshrc`). Open a new terminal, or source the profile named by the installer. For Fish or another unsupported/unknown shell, the installer does not edit profiles automatically; it prints the exact fallback command for the selected `INSTALL_DIR` (use `fish_add_path` in Fish or an `export PATH` command in a POSIX-compatible shell).

</details>

<details>
<summary><strong>Windows (PowerShell)</strong></summary>

```powershell
irm https://raw.githubusercontent.com/Soyagvs/shelly/main/install.ps1 | iex
```

</details>

> [!TIP]
> Prefer to inspect installation scripts first? Read [install.sh](install.sh) or [install.ps1](install.ps1), or download a binary from [Releases](https://github.com/Soyagvs/shelly/releases/latest).

### Release artifacts

Each release publishes one versioned archive per target: `shelly-<target>-<version>.tar.gz` (or `.zip` on Windows). Linux x86_64, macOS x86_64, and macOS aarch64 archives contain both `shelly` and `shelly-lsp`; every archive has a matching `.sha256` checksum file. Release builds use GitHub's supported `macos-15-intel` runner for macOS x86_64 and `macos-latest` for macOS arm64. A Mason registry package definition is prepared locally at [`packaging/mason-registry/packages/shelly-lsp/package.yaml`](packaging/mason-registry/packages/shelly-lsp/package.yaml); it has not been published or submitted upstream.

### Start a project

Create and enter a directory, initialize the scaffold, then run its entry point:

```sh
mkdir my-shelly-project
cd my-shelly-project
shelly init
shelly main.shelly
```

`init` always uses the current directory and accepts no project-name argument. It checks every scaffold target before writing; if any target already exists, it fails without changing files.

For a complete inventory of the generated files and directories, see [Project structure](docs/project-structure.md).

### Project configuration

`Shelly.toml` describes the project and its test discovery settings. The required `[project]` fields are `name`, `version`, and `entry`. `[test]` is optional; `directory` defaults to `tests` and `pattern` defaults to `*_test.shelly`.

```toml
[project]
name = "my-shelly-project"
version = "0.1.0"
entry = "main.shelly"

[test]
directory = "tests"
pattern = "*_test.shelly"
```

Run the manifest entry point from the project directory with `shelly run`. Shelly reads `Shelly.toml` from the current directory and resolves `[project].entry` relative to it:

```sh
shelly run
```

Validate the manifest entry without running it with the no-argument project check. It reads `Shelly.toml`, resolves `[project].entry`, and checks that script without executing its commands:

```sh
shelly check
```

You can also validate an explicit script path, or request JSON diagnostics:

```sh
shelly check path/to/script.shelly
shelly check --format json path/to/script.shelly
```

To forward arguments to the entry script, put `--` after `run`; every value after that separator is passed to the script:

```sh
shelly run -- --dry-run input.txt
```

The `--` separator is required before forwarded arguments. `shelly run --dry-run` is invalid because `--dry-run` is treated as an option to `shelly run`. The direct alternative remains available when you want to choose a script explicitly:

```sh
shelly main.shelly --dry-run input.txt
```

### Run tests

After `shelly init`, add a file such as **`tests/name_test.shelly`**. Tests are ordinary Shelly scripts: use normal control flow and finish with `exit 0` for success or a non-zero exit for failure.

```shelly
if $name != "Shelly"
    exit 1
end
exit 0
```

Run the test suite with:

```sh
shelly test
```

`shelly test` recursively discovers files named `*_test.shelly`, executes each in isolation, and prints a PASS/FAIL summary. It exits successfully only when every test passes; a failed test or test execution error makes the aggregate command fail.

### Test assertions

Use these assertion commands in `*_test.shelly` files:

```shelly
$value = "ready"
$items = ["one", "two"]
assertEqual $value "ready"
assertTrue true
assertFalse false
assertContains $items "two"
```

Assertions stop the test immediately when one fails. They are intended for test checks, not normal or recoverable program flow; use `if` for conditions and `try`/`catch` for recoverable errors.

Create **`hello.shelly`**:

````shelly
shellout "Hello from Shelly"

shellout ```
Hello
from multiple lines
```
````

Run it directly, or validate it first without executing commands:

```sh
# Optional validation
shelly check hello.shelly

# Run the script
shelly hello.shelly
```

## 🐦‍⬛ The language

Here's a taste of Shelly's syntax and its scripting features:

```shelly
$name = "Agustin"
$APP = "Shelly"
$names = ["Ana", "Luis", "Mora"]

shellout "Hello $name from $APP"
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

command pwd -> $cwd
command printf "warning" -> $out, $err, $code
shell "printf 'warning\\n' >&2; exit 7" -> $out, $err, $code
shellout "Running from $cwd"
shellout "last command code: $code stderr: $err"
exit 0
```

### How a script flows

```mermaid
flowchart LR
    A["📄 script.shelly"] --> D["▶ shelly script.shelly"]
    A -. Optional .-> B["🔎 shelly check"]
    B --> C{"Valid?"}
    C -->|Yes| D
    C -->|No| E["Fix script"]
    E --> B
    D --> F["Commands · Files · Data"]
    F --> G["Output · Exit status"]
    classDef purple fill:#2e1065,stroke:#a78bfa,color:#ffffff;
    classDef dark fill:#111827,stroke:#7c3aed,color:#ffffff;
    class A,B,D,F,G purple;
    class C,E dark;
```

*This is a usage workflow, not a diagram of Shelly's internal parser architecture.*

### Language at a glance

| Building blocks | What you can use |
| :--- | :--- |
| **Values** | Variables, constants, strings, lists, dictionaries, nested values |
| **Flow control** | `if`, `elif`, `else`, `case`, `repeat`, `while`, `for` |
| **Functions** | `call`, `return`, `break`, `continue` |
| **Conditions** | `and`, `or`, `not`, `exists`, `contains` |
| **Automation** | File helpers, logs, menus, commands, arguments, modules |
| **Processes** | Capture `stdout`, `stderr`, and exit code |
| **Structured data** | JSON helpers |

<details>
<summary><strong>Syntax essentials</strong></summary>

- `$name` — mutable variable; `$NAME` — immutable constant.
- `->` — bind a result to a variable.
- `end` — close a block.
- Commas separate normal arguments.
- List indexing starts at `1`.
- `exit 0` means success; `exit 1` means failure.

</details>

### Script arguments

```sh
shelly script.shelly Ada --dry-run
```

```shelly
arg 1 -> $name
hasArg "--dry-run" -> $dryrun
args -> $all
```

## 🧩 Editor support

| Editor / resource | Integration |
| :--- | :--- |
| **VS Code** | [Shelly VS Code extension](https://github.com/Soyagvs/shelly-vscode) — diagnostics and editing support |
| **Neovim** | [Syntax, icons, and Shelly LSP](editors/nvim) |
| **LSP** | [Shelly LSP v1](docs/lsp.md) — static workspace analysis, diagnostics, navigation, symbols, completion, hover, and safe rename |
| **File icons** | [Icon setup guide](docs/file-icons.md) |
| **Artwork** | [Shelly SVG](shelly.svg) |

## 📚 Documentation

**Start with the [Language Guide](docs/language.md)**, then jump to the topic you need.

For editor language features, see the [Shelly LSP guide](docs/lsp.md) and the [Neovim setup](editors/nvim).

<details>
<summary><strong>Language basics</strong></summary>

[Overview](docs/language.md) · [Comments](docs/comments.md) · [Variables](docs/variables.md) · [Constants](docs/constants.md) · [Strings](docs/strings.md) · [Numeric expressions](docs/numeric-expressions.md) · [Lists](docs/lists.md) · [Dictionaries](docs/dictionaries.md) · [JSON](docs/json.md)

</details>

<details>
<summary><strong>Control flow and functions</strong></summary>

[Conditionals](docs/conditionals.md) · [Predicates](docs/predicates.md) · [Repeat](docs/repeat.md) · [While](docs/while.md) · [For](docs/for.md) · [Functions](docs/functions.md) · [Error handling](docs/errors.md) · [Timeouts](docs/timeouts.md) · [Exit](docs/exit.md)

</details>

<details>
<summary><strong>Automation and tooling</strong></summary>

[Commands and input](docs/commands.md) · [Files](docs/files.md) · [Logging](docs/logging.md) · [Choose menus](docs/choose.md) · [Standard helpers](docs/helpers.md) · [Modules](docs/modules.md) · [Project structure](docs/project-structure.md) · [File icons](docs/file-icons.md)

</details>

## 📦 Platforms and development

<details>
<summary><strong>Manual downloads</strong></summary>

Get the appropriate archive from [Releases](https://github.com/Soyagvs/shelly/releases/latest):

| Platform | Asset |
| :--- | :--- |
| Linux x86_64 | `shelly-linux-x86_64.tar.gz` |
| macOS Intel | `shelly-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `shelly-macos-arm64.tar.gz` |
| Windows x86_64 | `shelly-windows-x86_64.zip` |

</details>

<details>
<summary><strong>Run from source and development checks</strong></summary>

```sh
cargo run -- path/to/script.shelly
cargo fmt --check
cargo test
```

</details>

---

<div align="center">

### Build something with Shelly.

**[Read the docs](docs/language.md)** · **[Try the latest release](https://github.com/Soyagvs/shelly/releases/latest)** · **[Explore the source](https://github.com/Soyagvs/shelly)**

<sub>Made with Rust · Licensed under <a href="LICENSE">GPL-3.0-or-later</a></sub>

</div>
