# VBWR B
#
# Project: UlanziDeck
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
#
# VibeWare: Human intent, AI execution, and plenty of tokens
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
#
# Modified with AI: OpenAI Codex; x64/ARM64 Store source packaging and private configuration; 2026-10-27
# Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
# Evidence: docs/CODE-REVIEW.md; current working-tree changes
#
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
#
# VBWR E

<#
.SYNOPSIS
Build Windows x64 or ARM64 MSIX packages from current source.
.DESCRIPTION
Local packages use the existing trusted signing certificate. -StoreConfig embeds
licensing configuration from an external JSON file and produces an unsigned Store
submission package. This script does not submit or publish to Partner Center.
.PARAMETER StoreConfig
Absolute path outside this repository to private Store identity and add-on JSON.
Normal Store submission packages remain unsigned.
.PARAMETER SignStoreTestPackage
Sign a Store-identity Debug package with an already trusted matching local certificate.
This enables local installation; it does not simulate a Store purchase or grant a license.
.PARAMETER Configuration
Debug or Release source compilation. Release is the distribution default.
.PARAMETER Architecture
Package architecture: x64 (default) or arm64. Cross-builds compile a host-native
settings generator from the same source; the target executable is never run.
.PARAMETER PackageVersion
Optional four-part MSIX version, independent of the source crate version. Store
versions require a nonzero major and a final zero (for example 1.0.0.0).
.PARAMETER PublisherSubject
Certificate subject for a local package. StoreConfig supplies the Store publisher.
.PARAMETER Install
Install the locally signed package, closing the existing app when needed.
.PARAMETER KeepStaging
Retain generated manifest, executables and assets for inspection.
.PARAMETER IconsOnly
Generate MSIX logo assets without building or packaging the application.
.PARAMETER DetailedPath
Optional detailed source logo path used by icon generation.
.PARAMETER CompactPath
Optional compact source logo path used for small icon sizes.
#>

[CmdletBinding()]
param(
    [switch] $IconsOnly,
    [string] $DetailedPath,
    [string] $CompactPath,
    [switch] $Install,
    [switch] $KeepStaging,
    [ValidateSet('Debug', 'Release')]
    [string] $Configuration = 'Release',
    [ValidateSet('x64', 'arm64')]
    [string] $Architecture = 'x64',
    [string] $PublisherSubject = 'CN=umber',
    [string] $StoreConfig,
    [string] $PackageVersion,
    [switch] $SignStoreTestPackage
)

if ($PSBoundParameters.Count -eq 0) {
    & (Join-Path $PSScriptRoot 'ulanzi.ps1')
    return
}

$ErrorActionPreference = 'Stop'
$Architecture = $Architecture.ToLowerInvariant()

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$workspacePrefix = $root.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar

$packageName = 'VibeWare.UlanziDock'
$packageDisplayName = 'UlanziDeck'
$publisherDisplayName = 'Umberto Giacobbi'
if ($SignStoreTestPackage -and (!$StoreConfig -or $Configuration -ne 'Debug')) {
    throw '-SignStoreTestPackage requires -StoreConfig and -Configuration Debug.'
}
if ($StoreConfig) {
    if ($Install -and !$SignStoreTestPackage) { throw 'Store submissions are unsigned; use a signed Debug test package for local installation.' }
    $StoreConfig = (Resolve-Path -LiteralPath $StoreConfig -ErrorAction Stop).ProviderPath
    if ($StoreConfig.StartsWith($workspacePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw 'Keep Store identity configuration outside the repository.'
    }
    $storeIdentity = Get-Content -LiteralPath $StoreConfig -Raw | ConvertFrom-Json
    foreach ($field in 'theme_product_id', 'package_family_name', 'package_name', 'publisher', 'publisher_display_name', 'display_name') {
        if ([string]::IsNullOrWhiteSpace([string]$storeIdentity.$field)) { throw "Store configuration is missing $field." }
    }
    $packageName = [string]$storeIdentity.package_name
    $PublisherSubject = [string]$storeIdentity.publisher
    $publisherDisplayName = [string]$storeIdentity.publisher_display_name
    $packageDisplayName = [string]$storeIdentity.display_name
    if ($packageDisplayName -cne $packageDisplayName.Trim()) { throw 'Store display_name must not contain surrounding whitespace.' }
}

# Path: candidate generated output; return its absolute path after workspace checks.
# Errors: paths outside the checkout or existing reparse points are rejected.
function Assert-WorkspacePath([string] $Path) {
    $full = [System.IO.Path]::GetFullPath($Path)
    if (-not $full.StartsWith($workspacePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Path is outside the project: $full"
    }
    if (Test-Path -LiteralPath $full) {
        $item = Get-Item -LiteralPath $full -Force
        if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Refusing a linked path: $full"
        }
    }
    return $full
}

# ToolName: SDK executable filename. Search installed SDK bin/x64 folders only.
# Errors: a missing tool stops packaging; nothing is downloaded or installed here.
function Get-WindowsSdkTool([string] $ToolName) {
    $roots = @(
        'C:\Program Files (x86)\Windows Kits\10\bin',
        'C:\Program Files\Windows Kits\10\bin'
    )
    $candidates = @(foreach ($sdkRoot in $roots) {
        if (Test-Path -LiteralPath $sdkRoot) {
            Get-ChildItem -LiteralPath $sdkRoot -Recurse -File -Filter $ToolName -ErrorAction SilentlyContinue |
                Where-Object { $_.DirectoryName -match '\\x64$' } |
                Sort-Object FullName -Descending |
                Select-Object -First 1
        }
    })
    $tool = $candidates | Select-Object -First 1
    if ($null -eq $tool) {
        throw "$ToolName was not found in the Windows SDK. Install the Windows App SDK tooling first."
    }
    return $tool.FullName
}

# Source: existing logo artwork; its pixels remain unchanged on disk.
# Destination: generated PNG path inside staging.
# Width: target canvas width in physical pixels.
# Height: target canvas height in physical pixels.
# Errors: image decoding/writing failures propagate; GDI resources are disposed.
function Save-Logo([string] $Source, [string] $Destination, [int] $Width, [int] $Height) {
    Add-Type -AssemblyName System.Drawing
    $image = [System.Drawing.Image]::FromFile($Source)
    $bitmap = New-Object System.Drawing.Bitmap $Width, $Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.Clear([System.Drawing.Color]::Transparent)
        $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
        $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
        $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::None
        $scale = [Math]::Min($Width / $image.Width, $Height / $image.Height) * 0.94
        $targetWidth = [Math]::Max(1, [int][Math]::Round($image.Width * $scale))
        $targetHeight = [Math]::Max(1, [int][Math]::Round($image.Height * $scale))
        $x = [int][Math]::Round(($Width - $targetWidth) / 2)
        $y = [int][Math]::Round(($Height - $targetHeight) / 2)
        $graphics.DrawImage($image, $x, $y, $targetWidth, $targetHeight)
        $bitmap.Save($Destination, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
        $image.Dispose()
    }
}

# Hex: HTML color string; return a System.Drawing color or propagate invalid input.
function Get-Color([string] $Hex) {
    return [System.Drawing.ColorTranslator]::FromHtml($Hex)
}

# Graphics: caller-owned destination surface; this function only disposes its brush.
# Hex: fill color.
# X: logical left coordinate, scaled fourfold into pixels.
# Y: logical top coordinate, scaled fourfold into pixels.
# Width: logical block width.
# Height: logical block height.
function Paint-Block(
    [System.Drawing.Graphics] $Graphics,
    [string] $Hex,
    [int] $X,
    [int] $Y,
    [int] $Width,
    [int] $Height
) {
    $brush = [System.Drawing.SolidBrush]::new((Get-Color $Hex))
    try {
        # Every logical pixel becomes a four-pixel block. This keeps the source
        # master sharp, and build-msix.ps1 subsequently uses nearest-neighbour.
        $Graphics.FillRectangle($brush, $X * 4, $Y * 4, $Width * 4, $Height * 4)
    }
    finally {
        $brush.Dispose()
    }
}

# Graphics: caller-owned canvas receiving one beveled key motif.
# X: logical key left; Y: logical key top.
# Width: logical key width; Height: logical key height.
# Face: main fill; Highlight: top edge; Shade: bottom edge; Sparkle: accent color.
function Paint-Key(
    [System.Drawing.Graphics] $Graphics,
    [int] $X,
    [int] $Y,
    [int] $Width,
    [int] $Height,
    [string] $Face,
    [string] $Highlight,
    [string] $Shade,
    [string] $Sparkle
) {
    Paint-Block $Graphics '#020a1c' $X $Y $Width $Height
    Paint-Block $Graphics $Face ($X + 4) ($Y + 4) ($Width - 8) ($Height - 8)
    Paint-Block $Graphics $Highlight ($X + 4) ($Y + 4) ($Width - 8) 7
    Paint-Block $Graphics $Shade ($X + 4) ($Y + $Height - 11) ($Width - 8) 7
    Paint-Block $Graphics $Sparkle ($X + 8) ($Y + 12) 7 7
}

# Graphics: caller-owned canvas receiving the fixed pixel-art frame.
function Paint-Frame([System.Drawing.Graphics] $Graphics) {
    Paint-Block $Graphics '#07162f' 0 0 256 256
    # The pad itself is the mark. The light stepped bezel keeps the VibeWare
    # family resemblance, but never takes space away from the coloured keys.
    Paint-Block $Graphics '#020715' 40 28 176 200
    Paint-Block $Graphics '#020715' 28 40 200 176
    Paint-Block $Graphics '#f7dfad' 40 20 176 200
    Paint-Block $Graphics '#f7dfad' 20 40 216 160
    Paint-Block $Graphics '#c9ab80' 40 204 176 16
    Paint-Block $Graphics '#c9ab80' 212 48 16 156
    Paint-Block $Graphics '#102b55' 40 40 176 164
    Paint-Block $Graphics '#274d7e' 48 40 160 8
    Paint-Block $Graphics '#06142d' 48 196 160 8
}

# Path: output PNG, never a source replacement unless explicitly selected by caller.
# Compact: choose the reduced-detail icon for small Windows sizes.
# Errors: image/file failures propagate; allocated drawing resources are disposed.
function New-Icon([string] $Path, [bool] $Compact) {
    $directory = Split-Path -Parent $Path
    [System.IO.Directory]::CreateDirectory($directory) | Out-Null
    $bitmap = [System.Drawing.Bitmap]::new(1024, 1024, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    try {
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        try {
            $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::None
            $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
            $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
            Paint-Frame $graphics
            if ($Compact) {
                Paint-Key $graphics 48 56 72 64 '#10cfc9' '#7afff3' '#078b9c' '#c0fffa'
                Paint-Key $graphics 136 56 72 64 '#ff9e30' '#ffda7a' '#d75019' '#fff1bb'
                Paint-Key $graphics 48 132 72 64 '#d947b9' '#ff9adc' '#8f277e' '#ffd0f0'
                Paint-Key $graphics 136 132 72 64 '#8657e8' '#c5aaff' '#4d32a1' '#eee6ff'
            }
            else {
                Paint-Key $graphics 48 48 48 48 '#10cfc9' '#7afff3' '#078b9c' '#c0fffa'
                Paint-Key $graphics 104 48 48 48 '#2c62ff' '#86a8ff' '#1938b7' '#d4e0ff'
                Paint-Key $graphics 160 48 48 48 '#ff9e30' '#ffda7a' '#d75019' '#fff1bb'
                Paint-Key $graphics 48 104 48 48 '#d947b9' '#ff9adc' '#8f277e' '#ffd0f0'
                Paint-Key $graphics 104 104 48 48 '#a3d549' '#dcff98' '#588d29' '#f1ffce'
                Paint-Key $graphics 160 104 48 48 '#8657e8' '#c5aaff' '#4d32a1' '#eee6ff'
                Paint-Key $graphics 48 160 48 32 '#00a9d6' '#81e9ff' '#0873a0' '#d4f8ff'
                Paint-Key $graphics 104 160 48 32 '#f05365' '#ffb0bd' '#a62b43' '#ffe2e8'
                Paint-Key $graphics 160 160 48 32 '#f1c84a' '#fff0a1' '#ad7a12' '#fff9d4'
            }
            $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
        }
        finally {
            $graphics.Dispose()
        }
    }
    finally {
        $bitmap.Dispose()
    }
}

# Generate detailed and compact logo sources using optional script-level paths.
# No compiler, package manager, certificate store or application process is invoked.
function Write-MsixIcons {
    Add-Type -AssemblyName System.Drawing
    if (-not $DetailedPath) { $DetailedPath = Join-Path $root 'assets/brand/ulanzi-dock-msix-icon.png' }
    if (-not $CompactPath) { $CompactPath = Join-Path $root 'assets/brand/ulanzi-dock-msix-icon-compact.png' }
    New-Icon $DetailedPath $false
    New-Icon $CompactPath $true
    Write-Host "Rendered MSIX icon masters: $DetailedPath and $CompactPath"
}

if ($IconsOnly) {
    Write-MsixIcons
    return
}

$cargo = Get-Command cargo -ErrorAction Stop
$targetTriple = if ($Architecture -eq 'arm64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
$rustVersion = & rustc -vV
if ($LASTEXITCODE -ne 0) { throw 'Could not read the Rust host target.' }
$hostLine = $rustVersion | Where-Object { $_ -match '^host: ' } | Select-Object -First 1
if (!$hostLine) { throw 'Rust did not report its host target.' }
$hostTriple = $hostLine.Substring(6).Trim()
$crossBuild = $targetTriple -ne $hostTriple
$makeAppx = Get-WindowsSdkTool 'makeappx.exe'
$signTool = if (!$StoreConfig -or $SignStoreTestPackage) { Get-WindowsSdkTool 'signtool.exe' }
$cargoToml = Join-Path $root 'Cargo.toml'
$versionMatch = Select-String -LiteralPath $cargoToml -Pattern '^version\s*=\s*"(?<version>[0-9]+\.[0-9]+\.[0-9]+)"' | Select-Object -First 1
if ($null -eq $versionMatch) {
    throw 'Cargo.toml does not contain a three-part package version.'
}
$version = "$($versionMatch.Matches[0].Groups['version'].Value).0"
if ($PackageVersion) {
    if ($PackageVersion -notmatch '^\d+\.\d+\.\d+\.\d+$') {
        throw 'PackageVersion must contain four integer sections, for example 1.0.0.0.'
    }
    $version = $PackageVersion
}
$parsedVersion = [version]$version
if (@($parsedVersion.Major, $parsedVersion.Minor, $parsedVersion.Build, $parsedVersion.Revision) |
    Where-Object { $_ -gt 65535 }) {
    throw 'Each MSIX version section must be between 0 and 65535.'
}
if ($StoreConfig -and !$SignStoreTestPackage -and ($parsedVersion.Major -lt 1 -or $parsedVersion.Revision -ne 0)) {
    throw 'Store packages require a nonzero major version and a final zero. Supply -PackageVersion, for example 1.0.0.0.'
}

$configurationSuffix = if ($Configuration -eq 'Debug') { '-debug' } else { '' }
$stage = Assert-WorkspacePath (Join-Path $root "tmp/msix-$Architecture$configurationSuffix")
$outputDir = Assert-WorkspacePath (Join-Path $root 'artifacts/msix')
$output = Assert-WorkspacePath (Join-Path $outputDir "ulanzi-dock-vibeware-$version-$Architecture$configurationSuffix.msix")
$unsigned = Assert-WorkspacePath (Join-Path $outputDir "ulanzi-dock-vibeware-$version-$Architecture$configurationSuffix.unsigned.msix")
$certificateFile = Assert-WorkspacePath (Join-Path $outputDir 'ulanzi-dock-vibeware-local.cer')

if (Test-Path -LiteralPath $stage) {
    Remove-Item -LiteralPath $stage -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $stage, $outputDir | Out-Null

try {
    Push-Location -LiteralPath $root
    $previousStoreConfig = $env:ULANZI_STORE_CONFIG
    $previousBuildTarget = $env:CARGO_BUILD_TARGET
    try {
        # Use the detected host for ordinary builds and an explicit target for
        # cross-builds, independent of a caller's CARGO_BUILD_TARGET setting.
        $env:CARGO_BUILD_TARGET = $hostTriple
        $buildArguments = @('build', '--locked', '--bins')
        if ($Configuration -eq 'Release') { $buildArguments += '--release' }
        if ($StoreConfig) {
            $env:ULANZI_STORE_CONFIG = $StoreConfig
            $buildArguments += @('--features', 'store-licensing')
        }
        $targetArguments = @($buildArguments)
        if ($crossBuild) { $targetArguments += @('--target', $targetTriple) }
        & $cargo.Source @targetArguments
        if ($LASTEXITCODE -ne 0) { throw "Cargo build failed with exit code $LASTEXITCODE." }
        if ($crossBuild) {
            & $cargo.Source @buildArguments
            if ($LASTEXITCODE -ne 0) { throw "Host settings-generator build failed with exit code $LASTEXITCODE." }
        }
    }
    finally {
        $env:ULANZI_STORE_CONFIG = $previousStoreConfig
        $env:CARGO_BUILD_TARGET = $previousBuildTarget
        Pop-Location
    }

    $buildOutput = Join-Path $root "target/$targetTriple/$($Configuration.ToLowerInvariant())"
    $seedOutput = Join-Path $root "target/$hostTriple/$($Configuration.ToLowerInvariant())"
    foreach ($binary in 'ulanzi-dock-vibeware.exe', 'ulanzi-dock-launcher.exe') {
        $source = Join-Path $buildOutput $binary
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
            throw "Cargo did not produce $source"
        }
        Copy-Item -LiteralPath $source -Destination (Join-Path $stage $binary) -Force
    }
    # Ship clean defaults, never the developer's personal settings or completed setup.
    & (Join-Path $seedOutput 'ulanzi-dock-vibeware.exe') init --config (Join-Path $stage 'settings.json')
    if ($LASTEXITCODE -ne 0) { throw 'Could not create the initial package settings.' }
    # The seeding process has exited; its empty lock sidecar is runtime state,
    # not package content. Remove only this known file within the staging folder.
    $seedLock = Join-Path $stage 'settings.json.lock'
    if (Test-Path -LiteralPath $seedLock) { Remove-Item -LiteralPath $seedLock }
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination (Join-Path $stage 'LICENSE') -Force
    Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination (Join-Path $stage 'README.md') -Force
    $notices = Join-Path $stage 'Notices'
    New-Item -ItemType Directory -Force -Path $notices | Out-Null
    Copy-Item -LiteralPath (Join-Path $root 'docs/THIRD-PARTY.md') -Destination (Join-Path $notices 'THIRD-PARTY.md') -Force
    Copy-Item -LiteralPath (Join-Path $root 'docs/licenses/slint-royalty-free-2.0.md') -Destination (Join-Path $notices 'Slint-LICENSE.md') -Force
    Copy-Item -LiteralPath (Join-Path $root 'assets/fonts/bitstream-vera-license.txt') -Destination (Join-Path $notices 'Bitstream-Vera-LICENSE.txt') -Force
    $noticeArguments = @((Join-Path $PSScriptRoot 'tools.py'), 'notices', '--mode', 'cargo', '--target', $targetTriple, '--output', (Join-Path $notices 'dependencies'))
    if ($StoreConfig) { $noticeArguments += @('--features', 'store-licensing') }
    & python @noticeArguments
    if ($LASTEXITCODE -ne 0) { throw "Dependency notice collection failed with exit code $LASTEXITCODE." }

    # Set XML values through the DOM so publisher names are escaped correctly.
    [xml]$manifest = Get-Content -LiteralPath (Join-Path $root 'packaging/msix/Package.appxmanifest.template') -Raw
    $manifest.Package.Identity.SetAttribute('Name', $packageName)
    $manifest.Package.Identity.SetAttribute('Version', $version)
    $manifest.Package.Identity.SetAttribute('Publisher', $PublisherSubject)
    $manifest.Package.Identity.SetAttribute('ProcessorArchitecture', $Architecture)
    $manifest.Package.Properties.PublisherDisplayName = $publisherDisplayName
    $manifest.Package.Properties.DisplayName = $packageDisplayName
    foreach ($node in $manifest.SelectNodes('//*[@DisplayName]')) {
        $node.SetAttribute('DisplayName', $packageDisplayName)
    }
    $manifest.Save((Join-Path $stage 'AppxManifest.xml'))
    $assets = Join-Path $stage 'Assets'
    New-Item -ItemType Directory -Force -Path $assets | Out-Null
    Write-MsixIcons
    $detailedLogo = Join-Path $root 'assets/brand/ulanzi-dock-msix-icon.png'
    $compactLogo = Join-Path $root 'assets/brand/ulanzi-dock-msix-icon-compact.png'
    foreach ($logo in $detailedLogo, $compactLogo) {
        if (-not (Test-Path -LiteralPath $logo -PathType Leaf)) {
            throw "The MSIX icon source is missing: $logo"
        }
    }
    $assetDefinitions = @(
        @{ Name = 'StoreLogo'; Width = 50; Height = 50 },
        @{ Name = 'Square71x71Logo'; Width = 71; Height = 71 },
        @{ Name = 'Square44x44Logo'; Width = 44; Height = 44 },
        @{ Name = 'Square150x150Logo'; Width = 150; Height = 150 },
        @{ Name = 'Square310x310Logo'; Width = 310; Height = 310 },
        @{ Name = 'Wide310x150Logo'; Width = 310; Height = 150 },
        @{ Name = 'SplashScreen'; Width = 620; Height = 300 }
    )
    foreach ($asset in $assetDefinitions) {
        $logo = if ([Math]::Max($asset.Width, $asset.Height) -lt 64) { $compactLogo } else { $detailedLogo }
        Save-Logo $logo (Join-Path $assets "$($asset.Name).png") $asset.Width $asset.Height
        foreach ($scale in 125, 150, 200, 400) {
            $width = [int][Math]::Round($asset.Width * $scale / 100)
            $height = [int][Math]::Round($asset.Height * $scale / 100)
            $logo = if ([Math]::Max($width, $height) -lt 64) { $compactLogo } else { $detailedLogo }
            Save-Logo $logo (Join-Path $assets "$($asset.Name).scale-$scale.png") $width $height
        }
    }
    foreach ($targetSize in 16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256) {
        $logo = if ($targetSize -lt 64) { $compactLogo } else { $detailedLogo }
        Save-Logo $logo (Join-Path $assets "Square44x44Logo.targetsize-$targetSize.png") $targetSize $targetSize
        foreach ($alternate in 'unplated', 'lightunplated') {
            Save-Logo $logo (Join-Path $assets "Square44x44Logo.targetsize-$targetSize`_altform-$alternate.png") $targetSize $targetSize
        }
    }

    # A Store submission must not remove an existing locally signed installer.
    $packageOutputs = if ($StoreConfig -and !$SignStoreTestPackage) { @($unsigned) } else { @($unsigned, $output) }
    foreach ($path in $packageOutputs) {
        if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
    }
    & $makeAppx pack /o /d $stage /p $unsigned
    if ($LASTEXITCODE -ne 0) { throw "MakeAppx failed with exit code $LASTEXITCODE." }

    if ($StoreConfig -and !$SignStoreTestPackage) {
        Write-Host "Unsigned Microsoft Store submission package: $unsigned"
        return
    }

    $certificate = Get-ChildItem -Path Cert:\CurrentUser\My |
        Where-Object { $_.Subject -eq $PublisherSubject -and $_.HasPrivateKey -and $_.NotAfter -gt (Get-Date).AddDays(30) } |
        Sort-Object NotAfter -Descending |
        Select-Object -First 1
    if ($null -eq $certificate) {
        throw "A usable signing certificate for $PublisherSubject is unavailable in Cert:\CurrentUser\My."
    }
    $trustedRoot = "Cert:\CurrentUser\Root\$($certificate.Thumbprint)"
    if (-not (Test-Path -LiteralPath $trustedRoot)) {
        throw "The signing certificate for $PublisherSubject is not trusted in Cert:\CurrentUser\Root."
    }
    Export-Certificate -Cert "Cert:\CurrentUser\My\$($certificate.Thumbprint)" -FilePath $certificateFile -Force | Out-Null
    & $signTool sign /fd SHA256 /sha1 $certificate.Thumbprint /s My $unsigned
    if ($LASTEXITCODE -ne 0) { throw "SignTool signing failed with exit code $LASTEXITCODE." }
    # Assert the signing identity. The package must use the current user's
    # pre-trusted local certificate; no certificate-store mutation occurs here.
    $signature = Get-AuthenticodeSignature -FilePath $unsigned
    if ($null -eq $signature.SignerCertificate -or $signature.SignerCertificate.Thumbprint -ne $certificate.Thumbprint) {
        throw 'The MSIX signer does not match the local UlanziDeck certificate.'
    }
    Move-Item -LiteralPath $unsigned -Destination $output -Force

    if ($Install) {
        Add-AppxPackage -Path $output -ForceApplicationShutdown -ForceUpdateFromAnyVersion
        $package = Get-AppxPackage -Name $packageName -ErrorAction Stop
        if ($package.Status -ne 'Ok') { throw "Installed MSIX has status $($package.Status)." }
        Write-Host "Installed: $($package.PackageFullName)"
    }
    Write-Host "MSIX ready: $output"
}
finally {
    if ((Test-Path -LiteralPath $stage) -and -not $KeepStaging) {
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
}
