#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/../../../.." && pwd)
VERSION=${1:-}
CHANNEL=${2:-stable}

[ -n "$VERSION" ] || { printf '%s\n' 'missing release version' >&2; exit 1; }
[ -n "${TAURI_DEV_RELEASES_PUBLIC_URL:-}" ] || { printf '%s\n' 'TAURI_DEV_RELEASES_PUBLIC_URL is required' >&2; exit 1; }

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM

export HOME="$tmpdir/home"
export TAURI_DEV_INSTALL_ROOT="$tmpdir/install"
export TAURI_DEV_LOCAL_BIN_DIR="$tmpdir/bin"
mkdir -p "$HOME" "$TAURI_DEV_INSTALL_ROOT" "$TAURI_DEV_LOCAL_BIN_DIR"

sh "$ROOT/scripts/manage/tauri-dev.sh" install --channel "$CHANNEL" --version "$VERSION"
"$TAURI_DEV_LOCAL_BIN_DIR/tauri-dev" --version
"$TAURI_DEV_LOCAL_BIN_DIR/tauri-dev" doctor --config "$ROOT/examples/minimal.toml"

if [ "${SMOKE_LATEST:-}" = "1" ]; then
  rm -f "$TAURI_DEV_LOCAL_BIN_DIR/tauri-dev"
  rm -rf "$TAURI_DEV_INSTALL_ROOT/latest-smoke"
  sh "$ROOT/scripts/manage/tauri-dev.sh" install --channel "$CHANNEL" --install-root "$TAURI_DEV_INSTALL_ROOT/latest-smoke"
  "$TAURI_DEV_LOCAL_BIN_DIR/tauri-dev" --version
  "$TAURI_DEV_LOCAL_BIN_DIR/tauri-dev" doctor --config "$ROOT/examples/minimal.toml"
fi

