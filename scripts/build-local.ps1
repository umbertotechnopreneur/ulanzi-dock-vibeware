# VBWR B
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# VibeWare is Human intent. AI implementation. Accountable human review.
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
# AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
# AI Versions: Tools and models may vary by contributor and execution environment.
# AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
# VBWR E

# Build in a dedicated temporary directory, publish one stable EXE, then
# remove this script's temporary output even when compilation fails.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw 'PowerShell 7 or newer is required.'
}

$project = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$workspacePrefix = $project.TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar

function Assert-WorkspacePath([string] $path) {
    $full = [System.IO.Path]::GetFullPath($path)
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

$temporaryParent = Assert-WorkspacePath (Join-Path $project 'tmp')
$artifactDir = Assert-WorkspacePath (Join-Path $project 'artifacts')
$buildDir = Assert-WorkspacePath (Join-Path $temporaryParent 'local-build')
$stableExe = Assert-WorkspacePath (Join-Path $artifactDir 'ulanzi-dock-vibeware.exe')
$pendingExe = Assert-WorkspacePath (Join-Path $artifactDir 'ulanzi-dock-vibeware.new.exe')
$previousExe = Assert-WorkspacePath (Join-Path $artifactDir 'ulanzi-dock-vibeware.previous.exe')

Get-Command cargo -ErrorAction Stop | Out-Null
New-Item -ItemType Directory -Force -Path $temporaryParent, $artifactDir | Out-Null

if (Test-Path -LiteralPath $previousExe) {
    if (Test-Path -LiteralPath $stableExe) {
        Remove-Item -LiteralPath $previousExe -Force
    }
    else {
        [System.IO.File]::Move($previousExe, $stableExe)
    }
}

if (Test-Path -LiteralPath $stableExe) {
    $running = Get-CimInstance Win32_Process -Filter "Name='ulanzi-dock-vibeware.exe'" |
        Where-Object { $_.ExecutablePath -ieq $stableExe }
    if ($running) {
        throw 'The stable EXE is running. Stop UlanziDock before replacing it.'
    }
}

try {
    if (Test-Path -LiteralPath $buildDir) {
        $buildDir = Assert-WorkspacePath $buildDir
        Remove-Item -LiteralPath $buildDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $buildDir | Out-Null

    Push-Location -LiteralPath $project
    try {
        & cargo build --release --target-dir $buildDir
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo build failed with exit code $LASTEXITCODE."
        }
    }
    finally {
        Pop-Location
    }

    $builtExe = Assert-WorkspacePath (Join-Path $buildDir 'release\ulanzi-dock-vibeware.exe')
    if (-not (Test-Path -LiteralPath $builtExe -PathType Leaf)) {
        throw "Cargo did not produce $builtExe"
    }
    Copy-Item -LiteralPath $builtExe -Destination $pendingExe -Force
    if (Test-Path -LiteralPath $stableExe) {
        [System.IO.File]::Replace($pendingExe, $stableExe, $previousExe)
        Remove-Item -LiteralPath $previousExe -Force
    }
    else {
        [System.IO.File]::Move($pendingExe, $stableExe)
    }
    Write-Output "Stable executable: $stableExe"
}
finally {
    if (Test-Path -LiteralPath $pendingExe) {
        $pendingExe = Assert-WorkspacePath $pendingExe
        Remove-Item -LiteralPath $pendingExe -Force
    }
    if (Test-Path -LiteralPath $buildDir) {
        $buildDir = Assert-WorkspacePath $buildDir
        Remove-Item -LiteralPath $buildDir -Recurse -Force
        Write-Output "Removed temporary build data: $buildDir"
    }
}
