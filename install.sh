#!/usr/bin/env sh
set -eu

REPO="Soyagvs/shelly"
VERSION="${SHELLY_VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

log() {
  printf '%s\n' "$1"
}

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

install_linux_file_icon() {
  [ "$os" = "Linux" ] || return 0

  data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
  mime_dir="$data_home/mime/packages"
  icon_dir="$data_home/icons/hicolor"

  mkdir -p "$mime_dir"
  cat > "$mime_dir/shelly.xml" <<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="text/x-shelly">
    <comment>Shelly script</comment>
    <glob pattern="*.shelly"/>
  </mime-type>
</mime-info>
XML

  mkdir -p "$icon_dir/scalable/mimetypes"
  if curl -fL "$icon_base_url/shelly.svg" -o "$icon_dir/scalable/mimetypes/text-x-shelly.svg" >/dev/null 2>&1; then
    log "Installed Linux Shelly SVG file icon."
  else
    log "Could not download Shelly SVG icon; continuing without file icon."
  fi

  for size in 16 32 48 128 256; do
    mkdir -p "$icon_dir/${size}x${size}/mimetypes"
    curl -fL "$icon_base_url/shelly-${size}.png" -o "$icon_dir/${size}x${size}/mimetypes/text-x-shelly.png" >/dev/null 2>&1 || true
  done

  if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database "$data_home/mime" >/dev/null 2>&1 || true
  fi
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t "$icon_dir" >/dev/null 2>&1 || true
  fi

  log "Registered .shelly MIME type for Linux desktop file icons."
}

need uname
need mktemp
need tar
need curl

os="$(uname -s)"
arch="$(uname -m)"

case "$os:$arch" in
  Linux:x86_64|Linux:amd64) target="linux-x86_64" ;;
  Darwin:x86_64) target="macos-x86_64" ;;
  Darwin:arm64|Darwin:aarch64) target="macos-arm64" ;;
  *) fail "unsupported platform: $os $arch" ;;
esac

if [ "$VERSION" = "latest" ]; then
  api_url="https://api.github.com/repos/$REPO/releases/latest"
  release_json="$(curl -fsSL "$api_url")" || fail "could not read the latest release tag from GitHub ($api_url)"
  tag="$(printf '%s' "$release_json" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\\([^"\\]*\\)".*/\\1/p' | head -n 1)"
  [ -n "$tag" ] || fail "GitHub latest release response did not contain a usable tag (check $api_url)"
  icon_ref="main"
else
  case "$VERSION" in
    v*) tag="$VERSION" ;;
    *) tag="v$VERSION" ;;
  esac
  icon_ref="$tag"
fi

case "$target" in
  linux-x86_64) archive="shelly-linux-x86_64-$tag.tar.gz" ;;
  macos-x86_64) archive="shelly-macos-x86_64-$tag.tar.gz" ;;
  macos-arm64) archive="shelly-macos-arm64-$tag.tar.gz" ;;
esac
url="https://github.com/$REPO/releases/download/$tag/$archive"
icon_base_url="https://raw.githubusercontent.com/$REPO/$icon_ref/assets/icons"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT INT TERM

log "Downloading Shelly from $url"
curl -fL "$url" -o "$tmpdir/$archive"

tar -xzf "$tmpdir/$archive" -C "$tmpdir"
[ -f "$tmpdir/shelly" ] || fail "downloaded archive did not contain shelly; cannot install the CLI"
[ -f "$tmpdir/shelly-lsp" ] || fail "downloaded archive did not contain shelly-lsp; cannot install the language server"
chmod +x "$tmpdir/shelly" "$tmpdir/shelly-lsp"

mkdir -p "$INSTALL_DIR"
cp "$tmpdir/shelly" "$INSTALL_DIR/shelly"
cp "$tmpdir/shelly-lsp" "$INSTALL_DIR/shelly-lsp"
chmod +x "$INSTALL_DIR/shelly" "$INSTALL_DIR/shelly-lsp"

log "Shelly installed to $INSTALL_DIR/shelly"
install_linux_file_icon

configure_path() {
  case ":${PATH:-}:" in
    *":$INSTALL_DIR:"*)
      log "Shelly is already on PATH. Run: shelly hello.shelly"
      return 0
      ;;
  esac

  shell_name="${SHELL##*/}"
  case "$shell_name" in
    bash)
      profile="$HOME/.bashrc"
      ;;
    zsh)
      profile="$HOME/.zshrc"
      ;;
    fish)
      log "Could not configure PATH automatically for Fish. Add Shelly with:"
      log "  fish_add_path \"$INSTALL_DIR\""
      log "Then open a new terminal or run that command, and run: shelly hello.shelly"
      return 0
      ;;
    *)
      log "Could not determine a supported shell profile. Add Shelly to PATH manually:"
      log "  export PATH=\"$INSTALL_DIR:\$PATH\""
      log "Then open a new terminal and run: shelly hello.shelly"
      return 0
      ;;
  esac

  if grep -Fq '# >>> shelly PATH >>>' "$profile" 2>/dev/null; then
    log "Shelly PATH setup already exists in $profile. Open a new terminal or run: . \"$profile\""
    log "Then run: shelly hello.shelly"
    return 0
  fi

  if printf '\n# >>> shelly PATH >>>\nexport PATH="%s:$PATH"\n# <<< shelly PATH <<<\n' "$INSTALL_DIR" >> "$profile" 2>/dev/null; then
    log "Added Shelly to PATH in $profile. Open a new terminal or run: . \"$profile\""
    log "Then run: shelly hello.shelly"
  else
    log "Could not create or update $profile; Shelly was installed successfully."
    log "Add Shelly to PATH manually, then open a new terminal:"
    log "  export PATH=\"$INSTALL_DIR:\$PATH\""
    log "Then run: shelly hello.shelly"
  fi
}

configure_path
