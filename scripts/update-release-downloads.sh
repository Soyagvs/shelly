#!/usr/bin/env bash
# Render the ten newest stable release download totals into README.md.
set -euo pipefail

start='<!-- release-downloads:start -->'
end='<!-- release-downloads:end -->'

releases_file=''
readme='README.md'
temp_releases=''
cleanup() { [[ -z "$temp_releases" ]] || rm -f "$temp_releases"; :; }
trap cleanup EXIT

if [[ $# -eq 2 ]]; then
  releases_file=$1
  readme=$2
elif [[ $# -eq 0 ]]; then
  temp_releases=$(mktemp)
  gh api --paginate "repos/${GITHUB_REPOSITORY}/releases?per_page=100" >"$temp_releases"
  releases_file=$temp_releases
else
  echo "usage: $0 [releases.json README.md]" >&2
  exit 2
fi

[[ -f "$releases_file" && -f "$readme" ]] || { echo 'release data and README must exist' >&2; exit 1; }

start_count=$(grep -Fc "$start" "$readme" || true)
end_count=$(grep -Fc "$end" "$readme" || true)
if [[ "$start_count" -ne 1 || "$end_count" -ne 1 ]]; then
  echo 'README must contain exactly one release-downloads marker pair' >&2
  exit 1
fi

rendered=$(mktemp)
updated=$(mktemp)
trap 'rm -f "$rendered" "$updated"; cleanup' EXIT
jq -r --arg start "$start" --arg end "$end" '
  $start,
  ([.[] | select(.prerelease == false)] | .[:10] | .[] |
    "- [\(.tag_name)](\(.html_url)): \([.assets[]?.download_count] | add // 0) downloads"),
  $end
' "$releases_file" >"$rendered"

awk -v start="$start" -v end="$end" -v replacement="$rendered" '
  $0 == start { if (seen_start++) exit 2; while ((getline line < replacement) > 0) print line; close(replacement); in_region=1; next }
  $0 == end { if (!in_region || seen_end++) exit 3; in_region=0; next }
  !in_region { print }
' "$readme" >"$updated" || { echo 'failed to replace README marker region' >&2; exit 1; }

# Keep the replacement byte-for-byte bounded; do not add a trailing newline after the end marker.
truncate -s -1 "$updated"
mv "$updated" "$readme"
