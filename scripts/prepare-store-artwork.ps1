# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# UlanziDeck - https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Created with AI: OpenAI Codex, owner-requested Microsoft Store artwork, 2026-10-27.
<#
.SYNOPSIS
Render Store listing artwork from the existing pixel-art icon.
.DESCRIPTION
Uses Windows drawing APIs only. Does not launch the app, access HID, download
dependencies or publish anything. Run build-msix.ps1 with -KeepStaging first.
The abstract hero contains no text, app UI or device photograph.
#>
[CmdletBinding()]
param([string] $OutputDirectory = 'artifacts/store-upload/1.0.3.0')
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$output = [IO.Path]::GetFullPath((Join-Path $root $OutputDirectory))
if (-not $output.StartsWith($root + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Output must be inside this checkout.'
}
foreach ($folder in 'listing', 'windows-icons', 'brand', 'screenshots') {
    [IO.Directory]::CreateDirectory((Join-Path $output $folder)) | Out-Null
}
$detailed = [Drawing.Image]::FromFile((Join-Path $root 'assets/brand/ulanzi-dock-msix-icon.png'))
$compact = [Drawing.Image]::FromFile((Join-Path $root 'assets/brand/ulanzi-dock-msix-icon-compact.png'))

function Draw-Icon($Graphics, $Image, [int]$X, [int]$Y, [int]$Size) {
    $Graphics.InterpolationMode = [Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $Graphics.PixelOffsetMode = [Drawing.Drawing2D.PixelOffsetMode]::Half
    $Graphics.DrawImage($Image, $X, $Y, $Size, $Size)
}

# A family of abstract key-colour compositions. No simulated app screenshots.
function Save-Art([string]$Name, [int]$Width, [int]$Height, [string]$Kind) {
    $bitmap = [Drawing.Bitmap]::new($Width, $Height)
    $g = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $background = [Drawing.Drawing2D.LinearGradientBrush]::new(
            [Drawing.Point]::new(0,0), [Drawing.Point]::new($Width,$Height),
            [Drawing.ColorTranslator]::FromHtml('#07162f'), [Drawing.ColorTranslator]::FromHtml('#19345a'))
        try { $g.FillRectangle($background, 0, 0, $Width, $Height) } finally { $background.Dispose() }
        if ($Kind -eq 'icon') {
            Draw-Icon $g $detailed 0 0 $Width
        }
        elseif ($Kind -eq 'hero') {
            $colors = @('#10cfc9','#2c62ff','#ff9e30','#d947b9','#a3d549','#8657e8','#00a9d6','#f05365','#f1c84a')
            $unit = [int]($Width / 24)
            $index = 0
            for ($row = 0; $row -lt 3; $row++) {
                for ($col = 0; $col -lt 5; $col++) {
                    $x = [int]($Width * .15) + $col * $unit * 3
                    $y = [int]($Height * .07) + $row * $unit * 2
                    $size = $unit + (($col + $row) % 2) * [int]($unit / 2)
                    $color = [Drawing.ColorTranslator]::FromHtml($colors[$index % $colors.Count])
                    $brush = [Drawing.SolidBrush]::new($color)
                    $edge = [Drawing.SolidBrush]::new([Drawing.Color]::FromArgb(100,255,255,255))
                    $shadow = [Drawing.SolidBrush]::new([Drawing.Color]::FromArgb(85,0,0,0))
                    try {
                        $g.FillRectangle($shadow, $x+8, $y+12, $size, $size)
                        $g.FillRectangle($brush, $x, $y, $size, $size)
                        $g.FillRectangle($edge, $x, $y, $size, [Math]::Max(4,[int]($unit/8)))
                    } finally { $brush.Dispose(); $edge.Dispose(); $shadow.Dispose() }
                    $index++
                }
            }
        }
        else {
            $size = [int]($Height * .64)
            Draw-Icon $g $detailed ([int]($Width*.07)) ([int]($Height*.12)) $size
            $title = [Drawing.Font]::new('Segoe UI', [single]($Height*.095), [Drawing.FontStyle]::Bold, [Drawing.GraphicsUnit]::Pixel)
            $body = [Drawing.Font]::new('Segoe UI', [single]($Height*.034), [Drawing.FontStyle]::Regular, [Drawing.GraphicsUnit]::Pixel)
            $g.TextRenderingHint = [Drawing.Text.TextRenderingHint]::AntiAliasGridFit
            try {
                $x = [single]($Width*.48)
                $g.DrawString('UlanziDeck', $title, [Drawing.Brushes]::White, $x, [single]($Height*.24))
                $g.DrawString('Your shortcuts. Your style.', $body, [Drawing.Brushes]::White, $x, [single]($Height*.43))
                $g.DrawString('Umberto Giacobbi', $body, [Drawing.Brushes]::LightSteelBlue, $x, [single]($Height*.57))
            } finally { $title.Dispose(); $body.Dispose() }
        }
        $bitmap.Save((Join-Path $output "listing/$Name.png"), [Drawing.Imaging.ImageFormat]::Png)
    } finally { $g.Dispose(); $bitmap.Dispose() }
}

try {
    Save-Art 'app-tile-300x300' 300 300 'icon'
    Save-Art 'app-tile-600x600' 600 600 'icon'
    Save-Art 'super-hero-1920x1080' 1920 1080 'hero'
    Save-Art 'super-hero-3840x2160' 3840 2160 'hero'
    Save-Art 'promotional-square-1080x1080' 1080 1080 'hero'
    Save-Art 'banner-titled-1920x1080' 1920 1080 'banner'
    Save-Art 'banner-titled-2400x1200' 2400 1200 'banner'
    $packageAssets = Join-Path $root 'tmp/msix-x64/Assets'
    if (!(Test-Path -LiteralPath $packageAssets)) { throw 'Build the MSIX with -KeepStaging first.' }
    Copy-Item -Path (Join-Path $packageAssets '*.png') -Destination (Join-Path $output 'windows-icons') -Force
    foreach ($name in 'ulanzi-dock-msix-icon.png','ulanzi-dock-msix-icon-compact.png','vibeware-logo.png','ulanzi-deck-logo.png') {
        Copy-Item -LiteralPath (Join-Path $root "assets/brand/$name") -Destination (Join-Path $output 'brand') -Force
    }
    # ICO stores PNG frames without requiring an external converter.
    $frames = @()
    foreach ($size in 16,24,32,48,64,128,256) {
        $bitmap = [Drawing.Bitmap]::new($size,$size)
        $g = [Drawing.Graphics]::FromImage($bitmap)
        $stream = [IO.MemoryStream]::new()
        try {
            $master = if ($size -lt 64) { $compact } else { $detailed }
            Draw-Icon $g $master 0 0 $size
            $bitmap.Save($stream,[Drawing.Imaging.ImageFormat]::Png)
            $frames += @{ Size=$size; Bytes=$stream.ToArray() }
        } finally { $stream.Dispose(); $g.Dispose(); $bitmap.Dispose() }
    }
    $file = [IO.File]::Create((Join-Path $output 'brand/ulanzi-dock.ico'))
    $writer = [IO.BinaryWriter]::new($file)
    try {
        $writer.Write([uint16]0); $writer.Write([uint16]1); $writer.Write([uint16]$frames.Count)
        $offset = 6 + 16*$frames.Count
        foreach ($frame in $frames) {
            $dimension = if ($frame.Size -eq 256) { 0 } else { $frame.Size }
            $writer.Write([byte]$dimension); $writer.Write([byte]$dimension)
            $writer.Write([byte]0); $writer.Write([byte]0)
            $writer.Write([uint16]1); $writer.Write([uint16]32)
            $writer.Write([uint32]$frame.Bytes.Length); $writer.Write([uint32]$offset)
            $offset += $frame.Bytes.Length
        }
        foreach ($frame in $frames) { $writer.Write([byte[]]$frame.Bytes) }
    } finally { $writer.Dispose(); $file.Dispose() }
} finally { $detailed.Dispose(); $compact.Dispose() }
Write-Host "Store artwork ready: $output"
