# Shelly — readable automation that scales

<img src="https://github.com/user-attachments/assets/975333ea-e6c7-421b-9795-089f07211ce6" alt="Shelly — raven logo" width="100%" />

Shelly is a modern scripting language for readable automation. Write clear scripts for files, system commands, structured data, interactive menus, and reusable modules in a fast Rust-based interpreter.

**Brand presentation:** Shelly uses purple as a visual identity in GitHub badges and documentation accents. Color is never the only signal: headings, labels, icons, and link text carry meaning too.

[![Release](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2FSoyagvs%2Fshelly%2Fmain%2F.github%2Fbadges%2Frelease.json&style=flat-square)](https://github.com/Soyagvs/shelly/releases)
[![Downloads](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2FSoyagvs%2Fshelly%2Fmain%2F.github%2Fbadges%2Fdownloads.json&style=flat-square)](https://github.com/Soyagvs/shelly/releases)
[![Stars](https://img.shields.io/github/stars/Soyagvs/shelly?style=flat-square&color=a78bfa)](https://github.com/Soyagvs/shelly/stargazers)
[![GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-6d28d9?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/built_with-Rust-242038?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

## ✦ Why Shelly?

- **Readable by design:** clear syntax for scripts that grow with your work.
- **Built for automation:** run commands, capture output and status, and work with files.
- **Structured and reusable:** use lists, dictionaries, JSON helpers, HTTP requests, and modules.

## ⚡ First success

Install Shelly, create a project, and run its entry script:

### Linux / macOS

```sh
curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Soyagvs/shelly/main/install.ps1 | iex
```

Then run:

```sh
mkdir my-shelly-project
cd my-shelly-project
shelly init
shelly main.shelly
```

Open a new terminal after installation if `shelly` is not yet on your `PATH`. On Windows, run `shelly --help` to see available commands.

See [Project structure](docs/project-structure.md) for an overview of the generated files.

## 🧭 Choose your path

- **New to Shelly:** follow the [Language Guide](docs/language.md).
- **Building an automation:** browse [Files](docs/files.md), [Archives](docs/archive.md), and [HTTP requests](docs/http.md).
- **Maintaining docs or integrations:** use the [documentation index](docs/index.md) and its maintainer guide.
- **Ready to contribute:** review the repository conventions and open an issue or pull request.

## 📚 Documentation

The [documentation index](docs/index.md) is the canonical map. It groups learning, standard-library, automation, and maintainer routes without changing the technical reference pages.

## 📦 Releases and license

Get the [latest release](https://github.com/Soyagvs/shelly/releases). Shelly is licensed under [GPL-3.0-or-later](LICENSE).
