#!/usr/bin/env bash
set -euo pipefail

[ -n "${R2_METADATA_URL:-}" ] || {
  echo "R2_METADATA_URL is required" >&2
  exit 1
}

tmpfile=$(mktemp)
trap 'rm -f "$tmpfile"' EXIT

curl -fsSL "$R2_METADATA_URL" -o "$tmpfile"
python3 -m json.tool "$tmpfile" >/dev/null
echo "R2 metadata ok: $R2_METADATA_URL"

