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
Build UlanziDeck or prepare its packages, pictures and notices.
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
.PARAMETER Architecture
For msix: choose x64 or arm64. The default is x64.
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
    [ValidateSet('preflight', 'setup', 'configure', 'run', 'deb', 'build', 'msix', 'package', 'icons', 'msix-icons', 'atlases', 'obs-guide', 'notices')]
    [string] $Command,
    [switch] $StopRunning,
    [switch] $KeepBuildCache,
    [switch] $Install,
    [switch] $KeepStaging,
    [ValidateSet('Debug', 'Release')] [string] $Configuration,
    [ValidateSet('x64', 'arm64')] [string] $Architecture,
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
    preflight = @()
    setup = @()
    configure = @()
    run = @()
    deb = @()
    build = @('StopRunning', 'KeepBuildCache')
    msix = @('Install', 'KeepStaging', 'Configuration', 'Architecture', 'PublisherSubject', 'StoreConfig', 'PackageVersion', 'SignStoreTestPackage')
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
    if ($Name -in @('preflight', 'setup', 'configure', 'run', 'deb') -or ($Name -eq 'build' -and -not $IsWindows)) {
        if ($Options.Count) { throw 'This task does not accept extra options on this platform.' }
        $python = if (Get-Command python3 -ErrorAction SilentlyContinue) { 'python3' } else { 'python' }
        & $python (Join-Path $PSScriptRoot 'ulanzi.py') $Name
        if ($LASTEXITCODE -ne 0) { throw "The $Name task failed (exit code $LASTEXITCODE)." }
        return
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
# All interactive navigation belongs to the shared cross-platform console.
$python = if (Get-Command python3 -ErrorAction SilentlyContinue) { 'python3' } else { 'python' }
& $python (Join-Path $PSScriptRoot 'ulanzi.py')
if ($LASTEXITCODE -ne 0) { throw "The console failed (exit code $LASTEXITCODE)." }
