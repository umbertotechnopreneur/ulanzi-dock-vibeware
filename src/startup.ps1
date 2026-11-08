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
#
# VBWR E

<#
.SYNOPSIS
Manage the current user's owned portable-app sign-in shortcut.
.DESCRIPTION
Called by startup.rs with ULANZIDOCK_STARTUP_MODE (status, enable or disable) and
ULANZIDOCK_STARTUP_EXE, _LAUNCHER, _WORKDIR and _CONFIG path values. Paths are passed as
data through the environment. The script checks ownership, stages changes and keeps
backups outside Startup. It never launches the app, changes drivers or touches HID.
Unrecognized shortcuts, malformed operations and file/COM failures stop the operation.
#>

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
$startupDirectory = [Environment]::GetFolderPath('Startup')
$shortcutPath = Join-Path $startupDirectory 'UlanziDock VibeWare.lnk'
$marker = 'UlanziDock VibeWare hidden startup entry'
$shell = New-Object -ComObject WScript.Shell
$existingShortcut = Test-Path -LiteralPath $shortcutPath
$ownedShortcut = $shell.CreateShortcut($shortcutPath)

# Never replace a same-named entry created by someone else.
if ($existingShortcut -and $ownedShortcut.Description -ne $marker) {
    throw "Preserving an unrecognized startup shortcut: $shortcutPath"
}

# Paths are passed directly to the native launcher. A trailing backslash must
# be doubled before the closing quote.
$shortcutArguments = (@($env:ULANZIDOCK_STARTUP_EXE, $env:ULANZIDOCK_STARTUP_WORKDIR, $env:ULANZIDOCK_STARTUP_CONFIG) | ForEach-Object {
    '"' + ($_ -replace '(\\+)$', '$1$1') + '"'
}) -join ' '
if ($env:ULANZIDOCK_STARTUP_MODE -eq 'status') {
    if (-not $existingShortcut) {
        Write-Output 'Launch at Windows sign-in: disabled'
        exit 0
    }
    if ($ownedShortcut.TargetPath -ine $env:ULANZIDOCK_STARTUP_LAUNCHER -or $ownedShortcut.Arguments -cne $shortcutArguments) {
        Write-Output "The sign-in shortcut points to another app or profile path. Run 'startup --enable' to use this copy."
    }
    elseif (-not (Test-Path -LiteralPath $ownedShortcut.TargetPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath $env:ULANZIDOCK_STARTUP_CONFIG -PathType Leaf)) {
        Write-Output "The launcher or settings file is missing. Restore it and run 'startup --enable'."
    }
    else {
        Write-Output 'The shortcut for a hidden launch at Windows sign-in is configured.'
        Write-Output "Settings: $env:ULANZIDOCK_STARTUP_CONFIG"
    }
    Write-Output "Entry: $shortcutPath"
    exit 0
}

# Keep recoverable copies outside Startup so backups cannot launch a second instance.
$backupDirectory = Join-Path $env:APPDATA 'UlanziDock VibeWare\Startup backups'
[IO.Directory]::CreateDirectory($backupDirectory) | Out-Null
$operationId = [Guid]::NewGuid().ToString('N')
$shortcutBackup = Join-Path $backupDirectory "$operationId.lnk"
if ($env:ULANZIDOCK_STARTUP_MODE -eq 'disable') {
    if ($existingShortcut) { [IO.File]::Move($shortcutPath, $shortcutBackup) }
    Write-Output 'Disabled launch at Windows sign-in for this user.'
    exit 0
}
if ($env:ULANZIDOCK_STARTUP_MODE -ne 'enable') { throw 'Unknown startup operation.' }

if ($shortcutArguments.Length -gt 1023) { throw 'Startup paths exceed the Windows shortcut argument limit; preserving the existing entry.' }
$stagedPath = Join-Path $backupDirectory "$operationId.new.lnk"
try {
    $shortcut = $shell.CreateShortcut($stagedPath)
    $shortcut.TargetPath = $env:ULANZIDOCK_STARTUP_LAUNCHER
    $shortcut.Arguments = $shortcutArguments
    $shortcut.WorkingDirectory = $env:ULANZIDOCK_STARTUP_WORKDIR
    $shortcut.IconLocation = $env:ULANZIDOCK_STARTUP_EXE + ',0'
    $shortcut.Description = $marker
    $shortcut.WindowStyle = 1
    $shortcut.Save()
    [Runtime.InteropServices.Marshal]::FinalReleaseComObject($shortcut) | Out-Null
    $shortcut = $null
    $savedShortcut = $shell.CreateShortcut($stagedPath)
    $savedArguments = $savedShortcut.Arguments
    [Runtime.InteropServices.Marshal]::FinalReleaseComObject($savedShortcut) | Out-Null
    if ($savedArguments -cne $shortcutArguments) {
        throw "Windows changed or truncated the startup command ($($shortcutArguments.Length) -> $($savedArguments.Length) characters); preserving the existing entry."
    }
    if ($existingShortcut) { [IO.File]::Replace($stagedPath, $shortcutPath, $shortcutBackup) }
    else { [IO.File]::Move($stagedPath, $shortcutPath) }

}
finally {
    if ([IO.File]::Exists($stagedPath)) { [IO.File]::Delete($stagedPath) }
}
Write-Output 'Enabled hidden tray launch at Windows sign-in for this user.'
Write-Output "Executable: $env:ULANZIDOCK_STARTUP_EXE"
Write-Output "Settings: $env:ULANZIDOCK_STARTUP_CONFIG"
Write-Output "Entry: $shortcutPath"
