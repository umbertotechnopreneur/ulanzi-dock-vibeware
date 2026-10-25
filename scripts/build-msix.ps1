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
    [switch] $IconsOnly,
    [string] $DetailedPath,
    [string] $CompactPath,
    [switch] $Install,
    [switch] $KeepStaging,
    [ValidateSet('Debug', 'Release')]
    [string] $Configuration = 'Release',
    [string] $PublisherSubject = 'CN=umber'
)

if ($PSBoundParameters.Count -eq 0) {
    & (Join-Path $PSScriptRoot 'ulanzi.ps1')
    return
}

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

function Get-Color([string] $Hex) {
    return [System.Drawing.ColorTranslator]::FromHtml($Hex)
}

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
    & python (Join-Path $PSScriptRoot 'tools.py') notices --mode cargo --target x86_64-pc-windows-msvc --output (Join-Path $notices 'dependencies')
    if ($LASTEXITCODE -ne 0) { throw "Dependency notice collection failed with exit code $LASTEXITCODE." }

    $manifest = (Get-Content -LiteralPath (Join-Path $root 'packaging/msix/Package.appxmanifest.template') -Raw).
        Replace('__MSIX_VERSION__', $version).
        Replace('__PUBLISHER__', $PublisherSubject)
    Set-Content -LiteralPath (Join-Path $stage 'AppxManifest.xml') -Value $manifest -Encoding utf8NoBOM
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
