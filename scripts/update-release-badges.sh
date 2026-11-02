#!/usr/bin/env bash
# Update repository-owned release badge payloads from the latest GitHub release.
set -euo pipefail

releases_json=$(mktemp)
trap 'rm -f "$releases_json" .github/badges/release.json.tmp .github/badges/downloads.json.tmp' EXIT

if gh api --paginate "repos/${GITHUB_REPOSITORY}/releases?per_page=100" >"$releases_json" 2>/dev/null; then
  readarray -t badge_values < <(jq -rs '
    [.[].[] | select(.draft == false and .prerelease == false)] as $releases |
    [($releases[]?.assets[]?.download_count // 0)] | add // 0,
    ($releases[0].tag_name // "unreleased")
  ' "$releases_json")
  downloads=${badge_values[0]}
  tag=${badge_values[1]}
else
  tag=unreleased
  downloads=0
fi

jq --arg message "$tag" '.message = $message' .github/badges/release.json > .github/badges/release.json.tmp
mv .github/badges/release.json.tmp .github/badges/release.json
jq --arg message "$downloads" '.message = $message' .github/badges/downloads.json > .github/badges/downloads.json.tmp
mv .github/badges/downloads.json.tmp .github/badges/downloads.json
