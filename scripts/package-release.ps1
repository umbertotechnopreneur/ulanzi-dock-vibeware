# VBWR B
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# VibeWare: Human intent. AI implementation. Accountable human review.
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
# Created with AI: OpenAI Codex; cross-platform release packaging, 2026-10-09.
# Human guidance: Umberto Giacobbi requested Windows, macOS and Linux packages.
# Evidence: Git history and GitHub Actions; physical device checks are separate.
# Copyright (c) 2026 Umberto Giacobbi
# License: MIT - see LICENSE
# VBWR E

[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string]$Target,
    [Parameter(Mandatory)]
    [ValidateSet('windows-x64', 'windows-arm64', 'linux-x64', 'macos-x64', 'macos-arm64')]
    [string]$Label,
    [ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$Tag = 'v0.1.0-stable'
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
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
Copy-Item -LiteralPath (Join-Path $root 'docs/ARTWORK.md'), (Join-Path $root 'docs/THIRD-PARTY.md'), (Join-Path $root 'docs/RELEASING.md'), (Join-Path $root 'docs/ulanzi-dock-vibeware-guide-v2.pdf') -Destination "$package/docs"
Copy-Item -Path (Join-Path $root 'docs/licenses/*') -Destination "$package/docs/licenses"
Copy-Item -LiteralPath (Join-Path $root 'assets/fonts/bitstream-vera-license.txt') -Destination "$package/assets/fonts"

& python (Join-Path $PSScriptRoot 'collect-notices.py') --mode cargo --target $Target --output (Join-Path $package 'third-party-notices')
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
