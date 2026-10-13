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
else
  case "$VERSION" in
    v*) tag="$VERSION" ;;
    *) tag="v$VERSION" ;;
  esac
  url="https://github.com/$REPO/releases/download/$tag/$archive"
fi

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
