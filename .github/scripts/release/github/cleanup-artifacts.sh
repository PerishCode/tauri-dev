#!/usr/bin/env bash
set -euo pipefail

[ -n "${GH_TOKEN:-}" ] || {
  echo "GH_TOKEN is required" >&2
  exit 1
}

run_id="${GITHUB_RUN_ID:-}"
[ -n "$run_id" ] || {
  echo "GITHUB_RUN_ID is required" >&2
  exit 1
}

gh api "repos/${GITHUB_REPOSITORY}/actions/runs/${run_id}/artifacts" \
  --jq '.artifacts[].id' |
while IFS= read -r artifact_id; do
  [ -n "$artifact_id" ] || continue
  gh api --method DELETE "repos/${GITHUB_REPOSITORY}/actions/artifacts/${artifact_id}"
done

