# VBWR B
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# VibeWare is Human intent, AI execution, and plenty of tokens
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# VBWR E

[CmdletBinding()]
param(
    [string] $DetailedPath,
    [string] $CompactPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
if ([string]::IsNullOrWhiteSpace($DetailedPath)) {
    $DetailedPath = Join-Path $root 'assets/brand/ulanzi-dock-msix-icon.png'
}
if ([string]::IsNullOrWhiteSpace($CompactPath)) {
    $CompactPath = Join-Path $root 'assets/brand/ulanzi-dock-msix-icon-compact.png'
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

New-Icon $DetailedPath $false
New-Icon $CompactPath $true
Write-Host "Rendered MSIX icon masters: $DetailedPath and $CompactPath"
