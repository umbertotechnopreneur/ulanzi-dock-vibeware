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
# remove this script's temporary output unless dependency caching is requested.
param(
    [switch] $StopRunning,
    [switch] $KeepBuildCache
)

$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw 'PowerShell 7 or newer is required.'
}

$project = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$workspacePrefix = $project.TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar

# path: a build or artifact path that must stay inside this workspace.
# Exceptions: the resolved path leaves the workspace or is a reparse point.
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

$running = Get-CimInstance Win32_Process -Filter "Name='ulanzi-dock-vibeware.exe'" |
    Where-Object {
        $_.ExecutablePath -and
        ([System.IO.Path]::GetFullPath($_.ExecutablePath)).StartsWith(
            $workspacePrefix, [System.StringComparison]::OrdinalIgnoreCase
        )
    }
if ($running -and -not $StopRunning) {
    throw 'UlanziDock is running in this workspace. Stop it or pass -StopRunning before building.'
}
foreach ($instance in $running) {
    $process = $null
    try {
        $process = [System.Diagnostics.Process]::GetProcessById($instance.ProcessId)
        # Recheck the live process path before killing, even if the snapshot PID was reused.
        if (-not $process.HasExited) {
            $actualPath = [System.IO.Path]::GetFullPath($process.MainModule.FileName)
            if ($actualPath -ine $instance.ExecutablePath) {
                throw "PID $($instance.ProcessId) changed executable; refusing to stop it."
            }
            Write-Output "Stopping UlanziDock PID $($instance.ProcessId): $actualPath"
            $process.Kill()
            if (-not $process.WaitForExit(5000)) {
                throw "UlanziDock PID $($instance.ProcessId) did not exit within five seconds."
            }
        }
    }
    catch [System.ArgumentException] {
        # The selected instance may exit between the snapshot and opening its process handle.
        Write-Output "UlanziDock PID $($instance.ProcessId) has already exited."
    }
    finally {
        if ($null -ne $process) {
            $process.Dispose()
        }
    }
}

try {
    if ((Test-Path -LiteralPath $buildDir) -and -not $KeepBuildCache) {
        $buildDir = Assert-WorkspacePath $buildDir
        Remove-Item -LiteralPath $buildDir -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $buildDir | Out-Null

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
    if ((Test-Path -LiteralPath $buildDir) -and -not $KeepBuildCache) {
        $buildDir = Assert-WorkspacePath $buildDir
        Remove-Item -LiteralPath $buildDir -Recurse -Force
        Write-Output "Removed temporary build data: $buildDir"
    }
}
