<div align="center">

<img src="https://github.com/user-attachments/assets/975333ea-e6c7-421b-9795-089f07211ce6" alt="Shelly — raven logo" width="100%" />

### A modern scripting language for readable automation.

**Write clear scripts. Automate real work. Go beyond Bash.**

[![Release](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2FSoyagvs%2Fshelly%2Fmain%2F.github%2Fbadges%2Frelease.json&style=flat-square)](https://github.com/Soyagvs/shelly/releases)
[![Downloads](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2FSoyagvs%2Fshelly%2Fmain%2F.github%2Fbadges%2Fdownloads.json&style=flat-square)](https://github.com/Soyagvs/shelly/releases)
[![Stars](https://img.shields.io/github/stars/Soyagvs/shelly?style=flat-square&color=a78bfa)](https://github.com/Soyagvs/shelly/stargazers)
[![GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-6d28d9?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/built_with-Rust-242038?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

[**Why Shelly?**](#-why-shelly) · [**Get started**](#-get-started) · [**Documentation**](#-documentation) · [**Releases**](https://github.com/Soyagvs/shelly/releases/latest)

</div>

---

## ✦ Why Shelly?

Shell scripts are useful, but they can become difficult to read and maintain. **Shelly** brings a clear, consistent language to everyday automation—files, system commands, structured data, interactive menus, and reusable modules—in a fast Rust-based interpreter.

- **Readable by design:** clear syntax for scripts that grow with your work.
- **Built for automation:** run commands, capture output and status, and work with files.
- **Structured and reusable:** use lists, dictionaries, JSON helpers, HTTP requests, and modules.

## ⚡ Get started

### Linux / macOS

```sh
curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh
```

Open a new terminal after installation if `shelly` is not yet on your `PATH`.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Soyagvs/shelly/main/install.ps1 | iex
```

Run `shelly --help` to see available commands.

### Start a project

```sh
mkdir my-shelly-project
cd my-shelly-project
shelly init
shelly main.shelly
```

See [Project structure](docs/project-structure.md) for an overview of the generated files.

## 📚 Documentation

Start with the [documentation index](docs/index.md) for the recommended reading path, or jump to the [Language Guide](docs/language.md) and [HTTP requests](docs/http.md).

---

<div align="center">

### Build something with Shelly.

**[Read the docs](docs/index.md)** · **[Try the latest release](https://github.com/Soyagvs/shelly/releases/latest)**

<sub>Made with Rust · Licensed under <a href="LICENSE">GPL-3.0-or-later</a></sub>

</div>
