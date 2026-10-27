# VBWR B
#
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
#
# VibeWare: Human intent, AI execution, and plenty of tokens
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
#
# Created with AI: OpenAI Codex; cross-platform release packaging, 2026-10-09.
# Modified with AI: OpenAI Codex; comments and readability; 2026-10-27
# Human guidance: Umberto Giacobbi; AGENTS.md; readability/documentation request
# Evidence: docs/CODE-REVIEW.md and current working-tree changes
#
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
#
# VBWR E

<#
.SYNOPSIS
Package a target already compiled from this checkout into a distribution archive.
.DESCRIPTION
Copies source-built binaries, documentation and local dependency notices into a fresh
staging folder, then writes an archive and checksum. Does not compile, run tests, install,
create a GitHub release or prove device behavior. Existing output names are preserved.
.PARAMETER Target
Rust target triple matching the selected Label and its source-build output directory.
.PARAMETER Label
Supported platform label used in filenames and archive-format selection.
.PARAMETER Tag
Version label used in the output archive name; this does not create a Git tag.
#>

[CmdletBinding()]
param(
    [string]$Target,
    [ValidateSet('windows-x64', 'windows-arm64', 'linux-x64', 'macos-arm64')]
    [string]$Label,
    [ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$Tag
)

if ($PSBoundParameters.Count -eq 0) {
    & (Join-Path $PSScriptRoot 'ulanzi.ps1')
    return
}
foreach ($name in 'Target', 'Label', 'Tag') {
    if (-not $PSBoundParameters.ContainsKey($name)) { throw "-$name is required. Run without parameters for the menu." }
}

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$targets = @{
    'windows-x64' = 'x86_64-pc-windows-msvc'
    'windows-arm64' = 'aarch64-pc-windows-msvc'
    'linux-x64' = 'x86_64-unknown-linux-gnu'
    'macos-arm64' = 'aarch64-apple-darwin'
}
if ($Target -ne $targets[$Label]) { throw "Target $Target does not match platform $Label." }
$package = Join-Path $root "dist/package-$Label"
$archiveRoot = Join-Path $root 'dist'
if (Test-Path -LiteralPath $package) { throw "Refusing to reuse a package directory: $package" }
New-Item -ItemType Directory -Path "$package/docs/licenses", "$package/assets/fonts" -Force | Out-Null
$isWindowsPackage = $Label.StartsWith('windows-')
$extension = if ($isWindowsPackage) { '.exe' } else { '' }
$binary = Join-Path $root "target/$Target/release/ulanzi-dock-vibeware$extension"
Copy-Item -LiteralPath $binary -Destination $package
if ($isWindowsPackage) {
    Copy-Item -LiteralPath (Join-Path $root "target/$Target/release/ulanzi-dock-launcher.exe") -Destination $package
}
Copy-Item -LiteralPath (Join-Path $root 'README.md'), (Join-Path $root 'LICENSE') -Destination $package
Copy-Item -LiteralPath (Join-Path $root 'docs/ARTWORK.md'), (Join-Path $root 'docs/THIRD-PARTY.md'), (Join-Path $root 'docs/RELEASING.md'), (Join-Path $root 'docs/INSTALLATION.md'), (Join-Path $root 'docs/PLATFORMS.md'), (Join-Path $root 'docs/README.md'), (Join-Path $root 'docs/BRANDING.md'), (Join-Path $root 'docs/DEVELOPMENT.md'), (Join-Path $root 'docs/ulanzi-dock-vibeware-guide-v2.pdf'), (Join-Path $root 'docs/obs-creator-quick-start.pdf') -Destination "$package/docs"
Copy-Item -LiteralPath (Join-Path $root 'docs/design') -Destination "$package/docs" -Recurse
Copy-Item -LiteralPath (Join-Path $root 'presets') -Destination $package -Recurse
Copy-Item -LiteralPath (Join-Path $root 'docs/photos'), (Join-Path $root 'docs/screenshots') -Destination "$package/docs" -Recurse
New-Item -ItemType Directory -Path "$package/assets/banners" | Out-Null
Copy-Item -LiteralPath (Join-Path $root 'assets/banners/ulanzi-dock-light.png') -Destination "$package/assets/banners"
Copy-Item -Path (Join-Path $root 'docs/licenses/*') -Destination "$package/docs/licenses"
Copy-Item -LiteralPath (Join-Path $root 'assets/fonts/bitstream-vera-license.txt') -Destination "$package/assets/fonts"

& python (Join-Path $PSScriptRoot 'tools.py') notices --mode cargo --target $Target --output (Join-Path $package 'third-party-notices')
if ($LASTEXITCODE -ne 0) { throw 'Dependency notice collection failed.' }

# Keep Unix executable permissions by using tar rather than a Windows-style ZIP.
$suffix = if ($isWindowsPackage) { 'zip' } else { 'tar.gz' }
$name = "ulanzi-dock-vibeware-$Tag-$Label.$suffix"
$archive = Join-Path $archiveRoot $name
if (Test-Path -LiteralPath $archive) { throw "Refusing to overwrite an existing archive: $archive" }
if ($isWindowsPackage) {
    Compress-Archive -Path "$package/*" -DestinationPath $archive
} else {
    & tar -czf $archive -C $package .
    if ($LASTEXITCODE -ne 0) { throw 'Archive creation failed.' }
}
$hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $name" | Set-Content -LiteralPath "$archive.sha256" -Encoding ascii
Write-Host "Package ready: $archive"
