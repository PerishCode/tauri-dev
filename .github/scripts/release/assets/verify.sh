#!/usr/bin/env sh
set -eu

mode=${1:-}
version=${2:-}
release_root=${3:-}

[ -n "$mode" ] || { echo "mode is required" >&2; exit 1; }
[ -n "$version" ] || { echo "release version is required" >&2; exit 1; }
[ -n "$release_root" ] || { echo "release root is required" >&2; exit 1; }

for name in \
  tauri-dev-x86_64-unknown-linux-gnu.tar.gz \
  tauri-dev-aarch64-apple-darwin.tar.gz \
  tauri-dev-x86_64-apple-darwin.tar.gz \
  tauri-dev-x86_64-pc-windows-msvc.zip \
  checksums.txt
do
  [ -f "$release_root/$name" ] || {
    echo "missing release asset: $release_root/$name" >&2
    exit 1
  }
done

if [ "$mode" = "verify" ]; then
  (
    cd "$release_root"
    shasum -a 256 -c checksums.txt
  )
elif [ "$mode" != "accept" ]; then
  echo "unsupported verify mode: $mode" >&2
  exit 1
fi

printf 'release assets %s for %s\n' "$mode" "$version"

