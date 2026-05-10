$ErrorActionPreference = 'Stop'

$command = if ($args.Length -gt 0) { $args[0] } else { 'install' }
$rest = if ($args.Length -gt 1) { $args[1..($args.Length - 1)] } else { @() }

$channel = if ($env:TAURI_DEV_CHANNEL) { $env:TAURI_DEV_CHANNEL } else { 'stable' }
$version = if ($env:TAURI_DEV_VERSION) { $env:TAURI_DEV_VERSION } else { '' }
$publicUrl = if ($env:TAURI_DEV_RELEASES_PUBLIC_URL) { $env:TAURI_DEV_RELEASES_PUBLIC_URL } else { '' }
$installRoot = if ($env:TAURI_DEV_INSTALL_ROOT) { $env:TAURI_DEV_INSTALL_ROOT } else { Join-Path $HOME '.local/share/tauri-dev' }
$localBinDir = if ($env:TAURI_DEV_LOCAL_BIN_DIR) { $env:TAURI_DEV_LOCAL_BIN_DIR } else { Join-Path $HOME '.local/bin' }

for ($i = 0; $i -lt $rest.Length; $i++) {
    switch -Regex ($rest[$i]) {
        '^--channel$' { $i++; $channel = $rest[$i]; continue }
        '^--channel=(.+)$' { $channel = $Matches[1]; continue }
        '^--version$' { $i++; $version = $rest[$i]; continue }
        '^--version=(.+)$' { $version = $Matches[1]; continue }
        '^--public-url$' { $i++; $publicUrl = $rest[$i]; continue }
        '^--public-url=(.+)$' { $publicUrl = $Matches[1]; continue }
        '^--install-root$' { $i++; $installRoot = $rest[$i]; continue }
        '^--install-root=(.+)$' { $installRoot = $Matches[1]; continue }
        '^--bin-dir$' { $i++; $localBinDir = $rest[$i]; continue }
        '^--bin-dir=(.+)$' { $localBinDir = $Matches[1]; continue }
        '^-h$|^--help$|^help$' {
            @'
tauri-dev installer

Usage:
  tauri-dev.ps1 install [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  tauri-dev.ps1 upgrade [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  tauri-dev.ps1 uninstall
'@ | Write-Output
            exit 0
        }
        default { throw "unknown argument: $($rest[$i])" }
    }
}

function Need-PublicUrl {
    if ([string]::IsNullOrWhiteSpace($publicUrl)) {
        throw 'TAURI_DEV_RELEASES_PUBLIC_URL or --public-url is required'
    }
    return $publicUrl.TrimEnd('/')
}

function Latest-Version($metadataPath) {
    $metadata = Get-Content -Raw -Path $metadataPath | ConvertFrom-Json
    return $metadata.releaseVersion
}

function Install-TauriDev {
    $baseUrl = Need-PublicUrl
    $tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("tauri-dev-install-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmpdir | Out-Null
    try {
        if ([string]::IsNullOrWhiteSpace($version)) {
            $metadataPath = Join-Path $tmpdir 'metadata.json'
            Invoke-WebRequest -Uri "$baseUrl/$channel/latest/metadata.json" -OutFile $metadataPath
            $script:version = Latest-Version $metadataPath
            if ([string]::IsNullOrWhiteSpace($script:version)) {
                throw 'failed to resolve latest tauri-dev version'
            }
        }

        $archive = 'tauri-dev-x86_64-pc-windows-msvc.zip'
        $archivePath = Join-Path $tmpdir $archive
        Invoke-WebRequest -Uri "$baseUrl/$channel/versions/$version/$archive" -OutFile $archivePath

        $versionRoot = Join-Path $installRoot $version
        New-Item -ItemType Directory -Force -Path $versionRoot | Out-Null
        New-Item -ItemType Directory -Force -Path $localBinDir | Out-Null
        Expand-Archive -LiteralPath $archivePath -DestinationPath $versionRoot -Force

        $cmd = Join-Path $localBinDir 'tauri-dev.cmd'
        $exe = Join-Path $versionRoot 'tauri-dev.exe'
        "@echo off`r`n`"$exe`" %*`r`n" | Set-Content -Encoding ASCII -Path $cmd
        & $cmd --version
        Write-Output "installed tauri-dev to $cmd"
    }
    finally {
        Remove-Item -LiteralPath $tmpdir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Uninstall-TauriDev {
    $cmd = Join-Path $localBinDir 'tauri-dev.cmd'
    Remove-Item -LiteralPath $cmd -Force -ErrorAction SilentlyContinue
    Write-Output "removed $cmd"
}

switch ($command) {
    'install' { Install-TauriDev }
    'upgrade' { Install-TauriDev }
    'uninstall' { Uninstall-TauriDev }
    default { throw "unknown command: $command" }
}

