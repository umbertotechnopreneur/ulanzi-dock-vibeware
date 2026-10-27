# VBWR B
#
# Project: UlanziDock VibeWare version
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
Build UlanziDock or prepare its packages, pictures and notices.
.DESCRIPTION
Without parameters, show the menu. With -Command, run that task directly.
Reject options that do not belong to the selected task. Work is delegated to the
source-build, packaging or artwork scripts; there is no hidden test or publish step.
.PARAMETER Command
Select the task. Other parameters are forwarded only when that task accepts them.
.PARAMETER StopRunning
For build: explicitly allow stopping verified workspace controller processes.
.PARAMETER KeepBuildCache
For build: keep the dedicated Cargo cache/output directory.
.PARAMETER Install
For msix: install the newly built package on this PC.
.PARAMETER KeepStaging
For msix: retain the package staging directory for inspection.
.PARAMETER Configuration
For msix: choose Debug or Release source compilation.
.PARAMETER PublisherSubject
For msix: select the existing local signing certificate by subject.
.PARAMETER StoreConfig
For msix: private external Store identity JSON, enabling Store licensing.
.PARAMETER PackageVersion
For msix: explicit four-part package version, for example 1.0.0.0 for the Store.
.PARAMETER SignStoreTestPackage
For msix: sign a Debug Store-identity package with the existing trusted certificate.
.PARAMETER Target
For package or notices: Rust target triple whose outputs/dependencies are selected.
.PARAMETER Label
For package: supported platform label used in archive names.
.PARAMETER Tag
For package: output version label, without creating a Git tag.
.PARAMETER Source
For atlases: directory containing the complete exported key pictures.
.PARAMETER Settings
For atlases: JSON settings used to produce the exported pictures.
.PARAMETER Output
For atlases or notices: destination directory.
.PARAMETER Artwork
For obs-guide: folder containing the released OBS key pictures.
.EXAMPLE
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command build -KeepBuildCache
.EXAMPLE
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command msix -Configuration Debug
#>
[CmdletBinding()]
param(
    [ValidateSet('build', 'msix', 'package', 'icons', 'msix-icons', 'atlases', 'obs-guide', 'notices')]
    [string] $Command,
    [switch] $StopRunning,
    [switch] $KeepBuildCache,
    [switch] $Install,
    [switch] $KeepStaging,
    [ValidateSet('Debug', 'Release')] [string] $Configuration,
    [string] $PublisherSubject,
    [string] $StoreConfig,
    [string] $PackageVersion,
    [switch] $SignStoreTestPackage,
    [string] $Target,
    [ValidateSet('windows-x64', 'windows-arm64', 'linux-x64', 'macos-arm64')] [string] $Label,
    [string] $Tag,
    [string] $Source,
    [string] $Settings,
    [string] $Output,
    [string] $Artwork
)

$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'PowerShell 7 or newer is required.' }
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$allowed = @{
    build = @('StopRunning', 'KeepBuildCache')
    msix = @('Install', 'KeepStaging', 'Configuration', 'PublisherSubject', 'StoreConfig', 'PackageVersion', 'SignStoreTestPackage')
    package = @('Target', 'Label', 'Tag')
    icons = @()
    'msix-icons' = @()
    atlases = @('Source', 'Settings', 'Output')
    'obs-guide' = @('Artwork')
    notices = @('Target', 'Output')
}

# Name: validated task name. Options: only that task's accepted argument/value pairs.
# Throws on unsupported options or a delegated command failure; does not swallow errors.
function Invoke-Task([string] $Name, [hashtable] $Options) {
    $Name = $Name.ToLowerInvariant()
    foreach ($key in $Options.Keys) {
        if ($key -notin $allowed[$Name]) { throw "-$key does not apply to $Name." }
    }
    if (-not $IsWindows -and $Name -in @('build', 'msix', 'msix-icons')) {
        throw 'This task uses Windows tools. On Linux or macOS, build with Cargo; see docs/DEVELOPMENT.md.'
    }
    switch ($Name) {
        build {
            # An explicit false switch bypasses the legacy command's empty-input menu.
            if ($Options.Count -eq 0) { $Options.KeepBuildCache = $false }
            & (Join-Path $PSScriptRoot 'build-local.ps1') @Options
        }
        msix {
            if ($Options.Count -eq 0) { $Options.Install = $false }
            & (Join-Path $PSScriptRoot 'build-msix.ps1') @Options
        }
        package {
            foreach ($key in 'Target', 'Label', 'Tag') {
                if (-not $Options[$key]) { throw "-$key is required for package." }
            }
            & (Join-Path $PSScriptRoot 'package-release.ps1') @Options
        }
        'msix-icons' { & (Join-Path $PSScriptRoot 'build-msix.ps1') -IconsOnly }
        default {
            if ($Name -eq 'atlases' -and -not $Options.Settings) { throw '-Settings is required for atlases.' }
            if ($Name -eq 'notices' -and -not $Options.Output) { throw '-Output is required for notices.' }
            $arguments = @((Join-Path $PSScriptRoot 'tools.py'), $Name)
            foreach ($key in $Options.Keys) { $arguments += @("--$($key.ToLowerInvariant())", $Options[$key]) }
            & python @arguments
            if ($LASTEXITCODE -ne 0) { throw "The $Name task failed (exit code $LASTEXITCODE)." }
        }
    }
}

# Prompt: user-facing label. Default: optional value returned for an empty answer.
# A required value is requested again until nonblank text is entered.
function Read-Value([string] $Prompt, [string] $Default = '') {
    $label = if ($Default) { "$Prompt [$Default]" } else { $Prompt }
    do { $value = Read-Host $label } while (-not $value.Trim() -and -not $Default)
    if ($value.Trim()) { return $value.Trim() }
    return $Default
}

if ($PSBoundParameters.Count -gt 0) {
    if (-not $Command) { throw 'Pass -Command with the task name, or run without parameters for the menu.' }
    $options = @{}
    foreach ($key in $PSBoundParameters.Keys) {
        if ($key -ne 'Command') { $options[$key] = $PSBoundParameters[$key] }
    }
    Invoke-Task $Command $options
    return
}
if ([Console]::IsInputRedirected) { throw 'Use -Command in unattended jobs. Run in a terminal for the menu.' }

Push-Location -LiteralPath $root
try {
    while ($true) {
        Write-Host "`nUlanziDock tools" -ForegroundColor Cyan
        Write-Host '  1  Build local Windows programs (Release, from current source)'
        Write-Host '  2  Build a Windows installer (Debug)'
        Write-Host '  3  Build and install on this PC (Debug; can close the running app)'
        Write-Host '  4  Make a ZIP or tar.gz from an already built source target'
        Write-Host '  5  Prepare app icons (PNG, ICO and ICNS)'
        Write-Host '  6  Prepare Windows installer icons'
        Write-Host '  7  Make preview sheets from exported buttons'
        Write-Host '  8  Rebuild the OBS preview and printable guide'
        Write-Host '  9  Collect dependency licenses from the local Cargo cache'
        Write-Host '  0  Exit'
        $choice = Read-Host 'Choose a number'
        if ($choice -eq '0' -or [string]::IsNullOrWhiteSpace($choice)) { break }
        try {
            switch ($choice.Trim()) {
                '1' { Invoke-Task 'build' @{} }
                { $_ -in '2', '3' } {
                    $subject = Read-Value 'Signing certificate subject' 'CN=umber'
                    Invoke-Task 'msix' @{ Configuration = 'Debug'; PublisherSubject = $subject; Install = ($choice.Trim() -eq '3') }
                }
                '4' {
                    $targets = @{
                        'windows-x64' = 'x86_64-pc-windows-msvc'; 'windows-arm64' = 'aarch64-pc-windows-msvc'
                        'linux-x64' = 'x86_64-unknown-linux-gnu'; 'macos-arm64' = 'aarch64-apple-darwin'
                    }
                    $label = Read-Value 'Platform: windows-x64, windows-arm64, linux-x64 or macos-arm64' 'windows-x64'
                    if (-not $targets.ContainsKey($label)) { throw 'Choose one of the four platform names.' }
                    $versionLine = Select-String -LiteralPath (Join-Path $root 'Cargo.toml') -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
                    $tag = Read-Value 'Archive tag' "v$($versionLine.Matches[0].Groups[1].Value)-stable"
                    Invoke-Task 'package' @{ Target = $targets[$label]; Label = $label; Tag = $tag }
                }
                '5' { Invoke-Task 'icons' @{} }
                '6' { Invoke-Task 'msix-icons' @{} }
                '7' {
                    $settings = Read-Value 'Settings file used to export the buttons'
                    $source = Read-Value 'Exported buttons folder' 'assets/buttons'
                    Invoke-Task 'atlases' @{ Settings = $settings; Source = $source }
                }
                '8' {
                    $artwork = Read-Value 'OBS exported artwork folder' 'tmp/obs-creator/artwork/dark-abstract/page-9'
                    Invoke-Task 'obs-guide' @{ Artwork = $artwork }
                }
                '9' {
                    $output = Read-Value 'Notices output folder' 'artifacts/third-party-notices'
                    Invoke-Task 'notices' @{ Output = $output }
                }
                default { Write-Host 'Choose a number from 0 to 9.' }
            }
        }
        catch { Write-Host $_.Exception.Message -ForegroundColor Red }
    }
}
finally { Pop-Location }
