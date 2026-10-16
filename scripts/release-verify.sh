#!/bin/sh
# Verify release archives contain both versioned Shelly executables.
set -eu

archive=${1:?usage: $0 ARCHIVE}
version=${2:?usage: $0 VERSION}
target=${3:?usage: $0 TARGET}

case "$archive" in
  *.tar.gz) tar -tzf "$archive" ;;
  *.zip) unzip -Z1 "$archive" ;;
  *) echo "unsupported archive: $archive" >&2; exit 2 ;;
esac | awk -v v="${version}" -v t="${target}" '
  $0 == "shelly" || $0 == "shelly.exe" { shelly=1 }
  $0 == "shelly-lsp" || $0 == "shelly-lsp.exe" { lsp=1 }
  END { if (!shelly || !lsp) { exit 1 } }
'
