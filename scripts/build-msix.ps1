# VBWR B
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# VibeWare is Human intent, AI execution, and plenty of tokens
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# VBWR E

[CmdletBinding()]
param(
    [switch] $Install,
    [switch] $KeepStaging,
    [ValidateSet('Debug', 'Release')]
    [string] $Configuration = 'Release',
    [string] $PublisherSubject = 'CN=umber'
)

$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$workspacePrefix = $root.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar

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

$cargo = Get-Command cargo -ErrorAction Stop
$makeAppx = Get-WindowsSdkTool 'makeappx.exe'
$signTool = Get-WindowsSdkTool 'signtool.exe'
$cargoToml = Join-Path $root 'Cargo.toml'
$versionMatch = Select-String -LiteralPath $cargoToml -Pattern '^version\s*=\s*"(?<version>[0-9]+\.[0-9]+\.[0-9]+)"' | Select-Object -First 1
if ($null -eq $versionMatch) {
    throw 'Cargo.toml does not contain a three-part package version.'
}
$version = "$($versionMatch.Matches[0].Groups['version'].Value).0"

$stage = Assert-WorkspacePath (Join-Path $root 'tmp/msix-x64')
$outputDir = Assert-WorkspacePath (Join-Path $root 'artifacts/msix')
$output = Assert-WorkspacePath (Join-Path $outputDir "ulanzi-dock-vibeware-$version-x64.msix")
$unsigned = Assert-WorkspacePath (Join-Path $outputDir "ulanzi-dock-vibeware-$version-x64.unsigned.msix")
$certificateFile = Assert-WorkspacePath (Join-Path $outputDir 'ulanzi-dock-vibeware-local.cer')

if (Test-Path -LiteralPath $stage) {
    Remove-Item -LiteralPath $stage -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $stage, $outputDir | Out-Null

try {
    Push-Location -LiteralPath $root
    try {
        $buildArguments = @('build', '--locked', '--bins')
        if ($Configuration -eq 'Release') { $buildArguments += '--release' }
        & $cargo.Source @buildArguments
        if ($LASTEXITCODE -ne 0) { throw "Cargo build failed with exit code $LASTEXITCODE." }
    }
    finally {
        Pop-Location
    }

    $buildOutput = Join-Path $root ('target/' + $Configuration.ToLowerInvariant())
    foreach ($binary in 'ulanzi-dock-vibeware.exe', 'ulanzi-dock-launcher.exe') {
        $source = Join-Path $buildOutput $binary
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
            throw "Cargo did not produce $source"
        }
        Copy-Item -LiteralPath $source -Destination (Join-Path $stage $binary) -Force
    }
    # Ship clean defaults, never the developer's personal settings or completed setup.
    & (Join-Path $buildOutput 'ulanzi-dock-vibeware.exe') init --config (Join-Path $stage 'settings.json')
    if ($LASTEXITCODE -ne 0) { throw 'Could not create the initial package settings.' }
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination (Join-Path $stage 'LICENSE') -Force
    Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination (Join-Path $stage 'README.md') -Force
    $notices = Join-Path $stage 'Notices'
    New-Item -ItemType Directory -Force -Path $notices | Out-Null
    Copy-Item -LiteralPath (Join-Path $root 'docs/THIRD-PARTY.md') -Destination (Join-Path $notices 'THIRD-PARTY.md') -Force
    Copy-Item -LiteralPath (Join-Path $root 'docs/licenses/slint-royalty-free-2.0.md') -Destination (Join-Path $notices 'Slint-LICENSE.md') -Force
    Copy-Item -LiteralPath (Join-Path $root 'assets/fonts/bitstream-vera-license.txt') -Destination (Join-Path $notices 'Bitstream-Vera-LICENSE.txt') -Force
    & python (Join-Path $PSScriptRoot 'collect-notices.py') --mode cargo --target x86_64-pc-windows-msvc --output (Join-Path $notices 'dependencies')
    if ($LASTEXITCODE -ne 0) { throw "Dependency notice collection failed with exit code $LASTEXITCODE." }

    $manifest = (Get-Content -LiteralPath (Join-Path $root 'packaging/msix/Package.appxmanifest.template') -Raw).
        Replace('__MSIX_VERSION__', $version).
        Replace('__PUBLISHER__', $PublisherSubject)
    Set-Content -LiteralPath (Join-Path $stage 'AppxManifest.xml') -Value $manifest -Encoding utf8NoBOM
    $assets = Join-Path $stage 'Assets'
    New-Item -ItemType Directory -Force -Path $assets | Out-Null
    & (Join-Path $PSScriptRoot 'render-msix-icons.ps1')
    if ($LASTEXITCODE -ne 0) { throw "MSIX icon rendering failed with exit code $LASTEXITCODE." }
    $detailedLogo = Join-Path $root 'assets/brand/ulanzi-dock-msix-icon.png'
    $compactLogo = Join-Path $root 'assets/brand/ulanzi-dock-msix-icon-compact.png'
    foreach ($logo in $detailedLogo, $compactLogo) {
        if (-not (Test-Path -LiteralPath $logo -PathType Leaf)) {
            throw "The MSIX icon source is missing: $logo"
        }
    }
    $assetDefinitions = @(
        @{ Name = 'StoreLogo'; Width = 50; Height = 50 },
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
    foreach ($targetSize in 16, 20, 24, 30, 32, 36, 40, 48) {
        Save-Logo $compactLogo (Join-Path $assets "Square44x44Logo.targetsize-$targetSize.png") $targetSize $targetSize
    }

    foreach ($path in $unsigned, $output) {
        if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
    }
    & $makeAppx pack /o /d $stage /p $unsigned
    if ($LASTEXITCODE -ne 0) { throw "MakeAppx failed with exit code $LASTEXITCODE." }

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
        throw 'The MSIX signer does not match the local UlanziDock certificate.'
    }
    Move-Item -LiteralPath $unsigned -Destination $output -Force

    if ($Install) {
        Add-AppxPackage -Path $output -ForceApplicationShutdown -ForceUpdateFromAnyVersion
        $package = Get-AppxPackage -Name 'VibeWare.UlanziDock' -ErrorAction Stop
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
