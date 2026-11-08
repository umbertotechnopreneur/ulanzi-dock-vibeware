# VBWR B
#
# Project: UlanziDeck
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
#
# VibeWare: Human intent, AI execution, and plenty of tokens
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
#
# Modified with AI: OpenAI Codex; comments and readability; 2026-10-27
# Human guidance: Umberto Giacobbi; AGENTS.md; readability/documentation request
# Evidence: docs/CODE-REVIEW.md and current working-tree changes
#
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
#
# VBWR E

<#
.SYNOPSIS
Build both Windows programs from the current source and replace workspace artifacts.
.DESCRIPTION
Uses a dedicated build directory and staged replacement with recoverable previous files.
It does not install, publish, run tests or contact the dock. Cleanup stays inside the
validated workspace paths. Cargo uses the locked dependencies already available on this machine.
.PARAMETER StopRunning
Explicitly allow stopping matching controller processes whose verified paths are in this workspace.
.PARAMETER KeepBuildCache
Retain the dedicated build directory between runs instead of removing it after the build.
#>

# Build in a dedicated temporary directory, publish one stable EXE, then
# remove this script's temporary output unless dependency caching is requested.
param(
    [switch] $StopRunning,
    [switch] $KeepBuildCache
)

if ($PSBoundParameters.Count -eq 0) {
    & (Join-Path $PSScriptRoot 'ulanzi.ps1')
    return
}

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
$pendingReplacement = Assert-WorkspacePath (Join-Path $artifactDir 'local-build.pending')
$executables = foreach ($name in 'ulanzi-dock-launcher', 'ulanzi-dock-vibeware') {
    @{
        Name = "$name.exe"
        Stable = Assert-WorkspacePath (Join-Path $artifactDir "$name.exe")
        Pending = Assert-WorkspacePath (Join-Path $artifactDir "$name.new.exe")
        Previous = Assert-WorkspacePath (Join-Path $artifactDir "$name.previous.exe")
    }
}

# Keep the backups until both restores finish so recovery itself can be retried.
function Restore-Executables {
    foreach ($file in $executables) {
        if ([IO.File]::Exists($file.Previous)) {
            [IO.File]::Copy($file.Previous, $file.Pending, $true)
            [IO.File]::Move($file.Pending, $file.Stable, $true)
        }
        else {
            [IO.File]::Delete($file.Stable)
        }
    }
    [IO.File]::Delete($pendingReplacement)
}

Get-Command cargo -ErrorAction Stop | Out-Null
New-Item -ItemType Directory -Force -Path $temporaryParent, $artifactDir | Out-Null
$lockPath = Assert-WorkspacePath (Join-Path $temporaryParent 'local-build.lock')
$buildLock = [IO.FileStream]::new($lockPath, 'OpenOrCreate', 'ReadWrite', 'None', 1, 'DeleteOnClose')
$cleanupStaging = $false
$ownsBuildDir = $false

try {
    $running = Get-CimInstance Win32_Process -Filter "Name='ulanzi-dock-vibeware.exe'" |
        Where-Object {
            $_.ExecutablePath -and
            ([System.IO.Path]::GetFullPath($_.ExecutablePath)).StartsWith(
                $workspacePrefix, [System.StringComparison]::OrdinalIgnoreCase
            )
        }
    if ($running -and -not $StopRunning) {
        throw 'UlanziDeck is running in this workspace. Stop it or pass -StopRunning before building.'
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
                Write-Output "Stopping UlanziDeck PID $($instance.ProcessId): $actualPath"
                $process.Kill()
                if (-not $process.WaitForExit(5000)) {
                    throw "UlanziDeck PID $($instance.ProcessId) did not exit within five seconds."
                }
            }
        }
        catch [System.ArgumentException] {
            # The selected instance may exit between the snapshot and opening its process handle.
            Write-Output "UlanziDeck PID $($instance.ProcessId) has already exited."
        }
        finally {
            if ($null -ne $process) {
                $process.Dispose()
            }
        }
    }
    if ([IO.File]::Exists($pendingReplacement)) {
        Restore-Executables
    }
    foreach ($file in $executables) {
        if ([IO.File]::Exists($file.Previous)) {
            if ([IO.File]::Exists($file.Stable)) {
                [IO.File]::Delete($file.Previous)
            }
            else {
                [IO.File]::Move($file.Previous, $file.Stable)
            }
        }
    }
    $cleanupStaging = $true
    $ownsBuildDir = $true
    if ((Test-Path -LiteralPath $buildDir) -and -not $KeepBuildCache) {
        $buildDir = Assert-WorkspacePath $buildDir
        Remove-Item -LiteralPath $buildDir -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $buildDir | Out-Null

    Push-Location -LiteralPath $project
    $previousAutoInstall = $env:RUSTUP_AUTO_INSTALL
    try {
        $env:RUSTUP_AUTO_INSTALL = '0'
        $rustVersion = & rustc -vV
        if ($LASTEXITCODE -ne 0) { throw 'Could not read the Rust host target.' }
        $hostLine = $rustVersion | Where-Object { $_ -match '^host: ' } | Select-Object -First 1
        if (!$hostLine) { throw 'Rust did not report its host target.' }
        $hostTriple = $hostLine.Substring(6).Trim()
        & cargo build --frozen --release --bins --target $hostTriple --target-dir $buildDir
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo build failed with exit code $LASTEXITCODE."
        }
    }
    finally {
        $env:RUSTUP_AUTO_INSTALL = $previousAutoInstall
        Pop-Location
    }

    foreach ($file in $executables) {
        $built = Assert-WorkspacePath (Join-Path $buildDir "$hostTriple/release/$($file.Name)")
        if (-not [IO.File]::Exists($built)) { throw "Cargo did not produce $built" }
        [IO.File]::Copy($built, $file.Pending, $true)
        if ([IO.File]::Exists($file.Stable)) {
            [IO.File]::Copy($file.Stable, $file.Previous)
        }
    }
    # A failed or interrupted pair replacement is rolled back from both backups.
    [IO.File]::Open($pendingReplacement, 'CreateNew').Dispose()
    try {
        foreach ($file in $executables) {
            [IO.File]::Move($file.Pending, $file.Stable, $true)
        }
        [IO.File]::Delete($pendingReplacement)
    }
    catch {
        Restore-Executables
        throw
    }
    Write-Output "Built the controller and launcher in $artifactDir"
}
finally {
    try {
        if ($cleanupStaging -and -not [IO.File]::Exists($pendingReplacement)) {
            foreach ($file in $executables) {
                [IO.File]::Delete($file.Pending)
                [IO.File]::Delete($file.Previous)
            }
        }
        if ($ownsBuildDir -and (Test-Path -LiteralPath $buildDir) -and -not $KeepBuildCache) {
            $buildDir = Assert-WorkspacePath $buildDir
            Remove-Item -LiteralPath $buildDir -Recurse -Force
        }
    }
    finally {
        $buildLock.Dispose()
    }
}
