# Install Shelly file icons

Use the platform installer to associate `.shelly` files with Shelly artwork. The installers provide desktop file associations only; they do not install a general editor theme or assign icons to arbitrary extensions.

## 🚀 Quick path

### Linux desktop

```sh
curl -fsSL https://raw.githubusercontent.com/Soyagvs/shelly/main/install.sh | sh
```

The installer registers the user-local `text-x-shelly` MIME type and installs available SVG/PNG assets in the icon theme.

### Windows Explorer

```powershell
irm https://raw.githubusercontent.com/Soyagvs/shelly/main/install.ps1 | iex
```

The installer registers `.shelly` for the current user and installs `shelly.ico` beside `shelly.exe`.

## Mapping and boundaries

| Platform | Mapping | Asset boundary |
| --- | --- | --- |
| Linux desktop | `.shelly` → `text-x-shelly` | SVG and PNG MIME icons |
| Windows Explorer | `.shelly` → `ShellyScript` | ICO association |
| macOS Finder | Not registered by the shell installer | Assets ship, but no app bundle, LaunchServices, or global Finder registration |

The source artwork is [`shelly.svg`](../shelly.svg). Generated installer assets are in [`assets/icons/`](../assets/icons/).

## Linux installer details

Files are written under `$XDG_DATA_HOME`, defaulting to `~/.local/share` when the variable is unset or empty:

```text
$XDG_DATA_HOME/mime/packages/shelly.xml
$XDG_DATA_HOME/icons/hicolor/*/mimetypes/text-x-shelly.*
```

A failed or unavailable PNG download is tolerated, so some sizes may be absent. `update-mime-database` and `gtk-update-icon-cache`, when available, refresh caches after installation; they do not install or govern the files.

## Windows installer details

The current-user registry entries are:

```text
HKCU\Software\Classes\.shelly
HKCU\Software\Classes\ShellyScript
```

The installer does not claim system-wide registration or editor-client integration.

## Related integrations and verification limits

The separate [Shelly VS Code extension](https://github.com/Soyagvs/shelly-vscode) repository documents VS Code integration. Other editor setup is covered by [Editor support](editor-support.md); this checkout does not bundle or verify those clients.

The Unix installer test covers PATH/profile behavior with a mocked download, but its archive fixture currently lacks `shelly-lsp`, which the installer requires, so those assertions cannot currently run. Tests do not perform end-to-end Linux desktop registration, validate downloaded icon contents, or exercise Windows/macOS icon registration.
