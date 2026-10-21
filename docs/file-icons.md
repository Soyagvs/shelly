<img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />

# File icons

Shelly includes artwork for `.shelly` script files.

## Mapping and assets

| File mapping | Asset(s) | Supported format boundary |
| --- | --- | --- |
| `.shelly` → `text-x-shelly` | [`shelly.svg`](../assets/icons/shelly.svg), `shelly-16/32/48/128/256.png` | Linux desktop MIME icons; SVG and PNG |
| `.shelly` → `ShellyScript` | [`shelly.ico`](../assets/icons/shelly.ico) | Windows Explorer association; ICO |

These assets describe `.shelly` files only; they do not assign icons to arbitrary extensions or provide a general editor theme.

## Source artwork

The source icon is [`../shelly.svg`](../shelly.svg). Generated installer assets live in [`../assets/icons`](../assets/icons).

## Linux desktop icons

The Linux installer registers a user-local MIME type for `.shelly` files and installs the Shelly icon into the user icon theme when desktop icon tools are available.

```sh
curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh
```

It writes user-local files under `$XDG_DATA_HOME`, defaulting to `~/.local/share` when `XDG_DATA_HOME` is unset or empty:

```text
$XDG_DATA_HOME/mime/packages/shelly.xml
$XDG_DATA_HOME/icons/hicolor/*/mimetypes/text-x-shelly.*
```

The installer tolerates a failed or unavailable PNG download and continues without that PNG. `update-mime-database` and `gtk-update-icon-cache`, when available, only refresh caches after installation; they do not install or govern the icon files.

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

## Supported clients and boundaries

The separate [Shelly VS Code extension](https://github.com/Soyagvs/shelly-vscode) repository is the documented location for VS Code editor integration; this checkout does not verify or bundle that integration. Other editor integrations are documented in [Editor support](editor-support.md); this checkout does not bundle their configuration.

The shell installers support Linux desktop MIME registration and Windows Explorer registration. The macOS shell installer does not register Finder icons, and no installer claims editor-client support.

## Installer-test coverage

The Unix installer test intends to cover PATH/profile behavior with a mocked download, but its archive fixture currently lacks `shelly-lsp`, which the installer requires; those PATH/profile assertions therefore cannot currently run. The tests do not perform end-to-end Linux desktop registration, validate downloaded icon contents, or exercise Windows and macOS icon registration.

<img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
