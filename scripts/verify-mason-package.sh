#!/usr/bin/env bash
set -euo pipefail

package="${1:-packaging/mason-registry/packages/shelly-lsp/package.yaml}"
[ -f "$package" ] || { echo "missing Mason package: $package" >&2; exit 1; }

required=(
  '^name: shelly-lsp$'
  '^source:$'
  '^  id: github$'
  '^  repo: Soyagvs/shelly$'
  '^  asset:$'
  '^    - target: linux_x64$'
  '^      file: shelly-linux-x86_64-\$\{version\}\.tar\.gz$'
  '^    - target: darwin_x64$'
  '^      file: shelly-macos-x86_64-\$\{version\}\.tar\.gz$'
  '^    - target: darwin_arm64$'
  '^      file: shelly-macos-arm64-\$\{version\}\.tar\.gz$'
  '^bin:$'
  '^  shelly-lsp: shelly-lsp$'
)
for pattern in "${required[@]}"; do
  grep -Eq "$pattern" "$package" || { echo "invalid Mason package (missing: $pattern)" >&2; exit 1; }
done

echo "valid Mason package definition: $package"
