$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))))
$version = if ($args.Length -gt 0) { $args[0] } else { '' }
$channel = if ($args.Length -gt 1) { $args[1] } else { 'stable' }

if ([string]::IsNullOrWhiteSpace($version)) {
    throw 'missing release version'
}
if ([string]::IsNullOrWhiteSpace($env:SIDECAR_RELEASES_PUBLIC_URL)) {
    throw 'SIDECAR_RELEASES_PUBLIC_URL is required'
}

$tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("sidecar-smoke-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpdir | Out-Null

try {
    $env:HOME = Join-Path $tmpdir 'home'
    $env:SIDECAR_INSTALL_ROOT = Join-Path $tmpdir 'install'
    $env:SIDECAR_LOCAL_BIN_DIR = Join-Path $tmpdir 'bin'
    New-Item -ItemType Directory -Force -Path $env:HOME, $env:SIDECAR_INSTALL_ROOT, $env:SIDECAR_LOCAL_BIN_DIR | Out-Null

    & "$root/scripts/manage/sidecar.ps1" install --channel $channel --version $version
    & (Join-Path $env:SIDECAR_LOCAL_BIN_DIR 'sidecar.cmd') --version
    & (Join-Path $env:SIDECAR_LOCAL_BIN_DIR 'sidecar.cmd') doctor --config (Join-Path $root 'examples/minimal.toml')

    if ($env:SMOKE_LATEST -eq '1') {
        Remove-Item -LiteralPath (Join-Path $env:SIDECAR_LOCAL_BIN_DIR 'sidecar.cmd') -Force -ErrorAction SilentlyContinue
        & "$root/scripts/manage/sidecar.ps1" install --channel $channel --install-root (Join-Path $env:SIDECAR_INSTALL_ROOT 'latest-smoke')
        & (Join-Path $env:SIDECAR_LOCAL_BIN_DIR 'sidecar.cmd') --version
        & (Join-Path $env:SIDECAR_LOCAL_BIN_DIR 'sidecar.cmd') doctor --config (Join-Path $root 'examples/minimal.toml')
    }
}
finally {
    Remove-Item -LiteralPath $tmpdir -Recurse -Force -ErrorAction SilentlyContinue
}
