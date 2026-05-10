$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))))
$version = if ($args.Length -gt 0) { $args[0] } else { '' }
$channel = if ($args.Length -gt 1) { $args[1] } else { 'stable' }

if ([string]::IsNullOrWhiteSpace($version)) {
    throw 'missing release version'
}
if ([string]::IsNullOrWhiteSpace($env:TAURI_DEV_RELEASES_PUBLIC_URL)) {
    throw 'TAURI_DEV_RELEASES_PUBLIC_URL is required'
}

$tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("tauri-dev-smoke-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpdir | Out-Null

try {
    $env:HOME = Join-Path $tmpdir 'home'
    $env:TAURI_DEV_INSTALL_ROOT = Join-Path $tmpdir 'install'
    $env:TAURI_DEV_LOCAL_BIN_DIR = Join-Path $tmpdir 'bin'
    New-Item -ItemType Directory -Force -Path $env:HOME, $env:TAURI_DEV_INSTALL_ROOT, $env:TAURI_DEV_LOCAL_BIN_DIR | Out-Null

    & "$root/scripts/manage/tauri-dev.ps1" install --channel $channel --version $version
    & (Join-Path $env:TAURI_DEV_LOCAL_BIN_DIR 'tauri-dev.cmd') --version
    & (Join-Path $env:TAURI_DEV_LOCAL_BIN_DIR 'tauri-dev.cmd') doctor --config (Join-Path $root 'examples/minimal.toml')

    if ($env:SMOKE_LATEST -eq '1') {
        Remove-Item -LiteralPath (Join-Path $env:TAURI_DEV_LOCAL_BIN_DIR 'tauri-dev.cmd') -Force -ErrorAction SilentlyContinue
        & "$root/scripts/manage/tauri-dev.ps1" install --channel $channel --install-root (Join-Path $env:TAURI_DEV_INSTALL_ROOT 'latest-smoke')
        & (Join-Path $env:TAURI_DEV_LOCAL_BIN_DIR 'tauri-dev.cmd') --version
        & (Join-Path $env:TAURI_DEV_LOCAL_BIN_DIR 'tauri-dev.cmd') doctor --config (Join-Path $root 'examples/minimal.toml')
    }
}
finally {
    Remove-Item -LiteralPath $tmpdir -Recurse -Force -ErrorAction SilentlyContinue
}

