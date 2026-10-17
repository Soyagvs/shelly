#!/usr/bin/env bash
# Update repository-owned release badge payloads from the latest GitHub release.
set -euo pipefail

release_json=$(mktemp)
trap 'rm -f "$release_json" .github/badges/release.json.tmp .github/badges/downloads.json.tmp' EXIT

if gh api "repos/${GITHUB_REPOSITORY}/releases/latest" >"$release_json" 2>/dev/null; then
  tag=$(jq -r '.tag_name' "$release_json")
  downloads=$(jq '[.assets[]?.download_count] | add // 0' "$release_json")
else
  tag=unreleased
  downloads=0
fi

jq --arg message "$tag" '.message = $message' .github/badges/release.json > .github/badges/release.json.tmp
mv .github/badges/release.json.tmp .github/badges/release.json
jq --arg message "$downloads" '.message = $message' .github/badges/downloads.json > .github/badges/downloads.json.tmp
mv .github/badges/downloads.json.tmp .github/badges/downloads.json
