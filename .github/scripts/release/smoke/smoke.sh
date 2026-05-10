#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/../../../.." && pwd)
VERSION=${1:-}
CHANNEL=${2:-stable}

[ -n "$VERSION" ] || { printf '%s\n' 'missing release version' >&2; exit 1; }
[ -n "${SIDECAR_RELEASES_PUBLIC_URL:-}" ] || { printf '%s\n' 'SIDECAR_RELEASES_PUBLIC_URL is required' >&2; exit 1; }

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM

export HOME="$tmpdir/home"
export SIDECAR_INSTALL_ROOT="$tmpdir/install"
export SIDECAR_LOCAL_BIN_DIR="$tmpdir/bin"
mkdir -p "$HOME" "$SIDECAR_INSTALL_ROOT" "$SIDECAR_LOCAL_BIN_DIR"

sh "$ROOT/scripts/manage/sidecar.sh" install --channel "$CHANNEL" --version "$VERSION"
"$SIDECAR_LOCAL_BIN_DIR/sidecar" --version
"$SIDECAR_LOCAL_BIN_DIR/sidecar" doctor --config "$ROOT/examples/minimal.toml"

if [ "${SMOKE_LATEST:-}" = "1" ]; then
  rm -f "$SIDECAR_LOCAL_BIN_DIR/sidecar"
  rm -rf "$SIDECAR_INSTALL_ROOT/latest-smoke"
  sh "$ROOT/scripts/manage/sidecar.sh" install --channel "$CHANNEL" --install-root "$SIDECAR_INSTALL_ROOT/latest-smoke"
  "$SIDECAR_LOCAL_BIN_DIR/sidecar" --version
  "$SIDECAR_LOCAL_BIN_DIR/sidecar" doctor --config "$ROOT/examples/minimal.toml"
fi
