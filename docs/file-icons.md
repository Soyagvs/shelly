# File icons

Shelly includes artwork for `.shelly` script files.

## Source artwork

The source icon is [`../shelly.svg`](../shelly.svg). Generated installer assets live in [`../assets/icons`](../assets/icons).

## Linux desktop icons

The Linux installer registers a user-local MIME type for `.shelly` files and installs the Shelly icon into the user icon theme when desktop icon tools are available.

```sh
curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh
```

It writes user-local files under:

```text
~/.local/share/mime/packages/shelly.xml
~/.local/share/icons/hicolor/*/mimetypes/text-x-shelly.*
```

Then it refreshes caches with `update-mime-database` and `gtk-update-icon-cache` when those commands exist.

## Windows file association

The Windows installer registers `.shelly` for the current user and points Explorer at the Shelly icon.

```powershell
irm https://raw.githubusercontent.com/Soyagvs/shelly/main/install.ps1 | iex
```

It writes under:

```text
HKCU\Software\Classes\.shelly
HKCU\Software\Classes\ShellyScript
```

The icon is installed next to `shelly.exe` as `shelly.ico`.

## macOS note

The current shell installer installs the `shelly` command-line binary. macOS file icons for arbitrary extensions are normally registered through an app bundle, LaunchServices, or a `.pkg` installer. Shelly ships the icon assets, but the current shell installer does not globally register a Finder icon for `.shelly` files yet.

## VS Code

The separate VS Code extension includes the Shelly icon for editor file-icon support:

<https://github.com/Soyagvs/shelly-vscode>
