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
if ($env:ULANZIDOCK_STARTUP_MODE -eq 'status') {
    if ($existingShortcut) {
        Write-Output "Launch at Windows sign-in: enabled (hidden tray launch)"
        Write-Output "Entry: $shortcutPath"
    }
    else { Write-Output 'Launch at Windows sign-in: disabled' }
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

# Native argument quoting: each path is data, with no shell expansion. Double quotes
# were rejected by the caller. Doubling a trailing backslash protects the closing quote.
$shortcutArguments = (@($env:ULANZIDOCK_STARTUP_EXE, $env:ULANZIDOCK_STARTUP_WORKDIR, $env:ULANZIDOCK_STARTUP_CONFIG) | ForEach-Object {
    '"' + ($_ -replace '(\\+)$', '$1$1') + '"'
}) -join ' '
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
Write-Output "Entry: $shortcutPath"
