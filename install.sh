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
  Linux:x86_64|Linux:amd64)
    archive="shelly-linux-x86_64.tar.gz"
    ;;
  Darwin:x86_64)
    archive="shelly-macos-x86_64.tar.gz"
    ;;
  Darwin:arm64|Darwin:aarch64)
    archive="shelly-macos-arm64.tar.gz"
    ;;
  *)
    fail "unsupported platform: $os $arch"
    ;;
esac

if [ "$VERSION" = "latest" ]; then
  url="https://github.com/$REPO/releases/latest/download/$archive"
  icon_ref="main"
else
  case "$VERSION" in
    v*) tag="$VERSION" ;;
    *) tag="v$VERSION" ;;
  esac
  url="https://github.com/$REPO/releases/download/$tag/$archive"
  icon_ref="$tag"
fi
icon_base_url="https://raw.githubusercontent.com/$REPO/$icon_ref/assets/icons"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT INT TERM

log "Downloading Shelly from $url"
curl -fL "$url" -o "$tmpdir/$archive"

tar -xzf "$tmpdir/$archive" -C "$tmpdir"
[ -f "$tmpdir/shelly" ] || fail "downloaded archive did not contain shelly"
chmod +x "$tmpdir/shelly"

mkdir -p "$INSTALL_DIR"
cp "$tmpdir/shelly" "$INSTALL_DIR/shelly"
chmod +x "$INSTALL_DIR/shelly"

log "Shelly installed to $INSTALL_DIR/shelly"
install_linux_file_icon

case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    log "Run: shelly hello.shelly"
    ;;
  *)
    log "Add this to your shell profile if 'shelly' is not found:"
    log "  export PATH=\"$INSTALL_DIR:\$PATH\""
    log "Then run: shelly hello.shelly"
    ;;
esac
