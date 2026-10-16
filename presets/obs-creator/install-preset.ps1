# VBWR B
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# VibeWare is Human intent, AI execution, and plenty of tokens
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
# AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
# AI Versions: Tools and models may vary by contributor and execution environment.
# AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
# VBWR E

#Requires -Version 7.0

<#
.SYNOPSIS
Adds the optional OBS Creator page to an existing UlanziDock settings file.
.DESCRIPTION
Close UlanziDock before using this script. An explicit settings path is required.
Existing page values and unknown JSON values are retained as raw JSON. The nine
standard pages keep their canonical order, additional custom pages are retained,
and OBS Creator is placed immediately before the final Themes page.
The script never starts UlanziDock, OBS, a build, or a device session. It does not
change applications.yaml. OBS hotkeys must be configured manually.
.PARAMETER SettingsPath
The existing settings.json used by your UlanziDock installation.
.PARAMETER Replace
Explicitly replace an existing OBS Creator page, including its customizations.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string] $SettingsPath,
    [switch] $Replace
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Assert-UniqueJsonProperties {
    param([System.Text.Json.JsonElement] $Element, [string] $Location)
    if ($Element.ValueKind -eq [System.Text.Json.JsonValueKind]::Object) {
        $names = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
        foreach ($property in $Element.EnumerateObject()) {
            if (-not $names.Add($property.Name)) {
                throw "Duplicate JSON property '$($property.Name)' at $Location."
            }
            Assert-UniqueJsonProperties $property.Value "$Location.$($property.Name)"
        }
    } elseif ($Element.ValueKind -eq [System.Text.Json.JsonValueKind]::Array) {
        $index = 0
        foreach ($item in $Element.EnumerateArray()) {
            Assert-UniqueJsonProperties $item "$Location[$index]"
            $index++
        }
    }
}

function Assert-Page {
    param([System.Text.Json.JsonElement] $Page)
    if ($Page.ValueKind -ne [System.Text.Json.JsonValueKind]::Object) {
        throw 'Each page must be a JSON object.'
    }
    $name = $Page.GetProperty('name')
    if ($name.ValueKind -ne [System.Text.Json.JsonValueKind]::String -or
        [string]::IsNullOrWhiteSpace($name.GetString())) {
        throw 'Each page needs a nonempty string name.'
    }
    $optional = [System.Text.Json.JsonElement]::new()
    if ($Page.TryGetProperty('enabled', [ref] $optional) -and
        $optional.ValueKind -notin @([System.Text.Json.JsonValueKind]::True, [System.Text.Json.JsonValueKind]::False)) {
        throw "Page '$($name.GetString())' has an invalid enabled value."
    }
    if ($Page.TryGetProperty('executables', [ref] $optional)) {
        if ($optional.ValueKind -ne [System.Text.Json.JsonValueKind]::Array) {
            throw "Page '$($name.GetString())' executables must be an array."
        }
        foreach ($executable in $optional.EnumerateArray()) {
            if ($executable.ValueKind -ne [System.Text.Json.JsonValueKind]::String) {
                throw "Page '$($name.GetString())' has a nonstring executable."
            }
        }
    }
    $keys = $Page.GetProperty('keys')
    if ($keys.ValueKind -ne [System.Text.Json.JsonValueKind]::Array -or $keys.GetArrayLength() -ne 14) {
        throw "Page '$($name.GetString())' must contain exactly 14 physical keys."
    }
    foreach ($key in $keys.EnumerateArray()) {
        if ($key.ValueKind -ne [System.Text.Json.JsonValueKind]::Object -or
            $key.GetProperty('label').ValueKind -ne [System.Text.Json.JsonValueKind]::String -or
            $key.GetProperty('action').ValueKind -ne [System.Text.Json.JsonValueKind]::String) {
            throw "Page '$($name.GetString())' contains an invalid key."
        }
        foreach ($field in @('description', 'asset')) {
            if ($key.TryGetProperty($field, [ref] $optional) -and
                $optional.ValueKind -ne [System.Text.Json.JsonValueKind]::String) {
                throw "Page '$($name.GetString())' has an invalid $field."
            }
        }
    }
    if ($keys[4].GetProperty('action').GetString() -cne 'page:next' -or
        $keys[13].GetProperty('action').GetString() -cne 'none') {
        throw "Page '$($name.GetString())' must retain navigation at key 4 and no action at key 13."
    }
}

$settingsItem = Get-Item -LiteralPath $SettingsPath
if ($settingsItem -isnot [System.IO.FileInfo] -or
    ($settingsItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint)) {
    throw 'SettingsPath must identify an existing regular settings file, not a directory or symbolic link.'
}
$targetPath = $settingsItem.FullName
$presetPath = Join-Path $PSScriptRoot 'page.json'
if ([string]::Equals($targetPath, $presetPath, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'SettingsPath must point to your application settings, not the bundled page.json.'
}
$originalBytes = [System.IO.File]::ReadAllBytes($targetPath)
$originalFingerprint = [Convert]::ToBase64String($originalBytes)
$settingsText = [System.IO.File]::ReadAllText($targetPath)
$settingsDocument = $null
$presetDocument = $null
$candidateDocument = $null
$temporaryPath = $null

try {
    $settingsDocument = [System.Text.Json.JsonDocument]::Parse($settingsText)
    $presetDocument = [System.Text.Json.JsonDocument]::Parse([System.IO.File]::ReadAllText($presetPath))
    $root = $settingsDocument.RootElement
    if ($root.ValueKind -ne [System.Text.Json.JsonValueKind]::Object) {
        throw 'The existing settings must be a JSON object.'
    }
    Assert-UniqueJsonProperties $root 'settings'
    Assert-UniqueJsonProperties $presetDocument.RootElement 'preset'
    Assert-Page $presetDocument.RootElement
    if ($presetDocument.RootElement.GetProperty('name').GetString() -cne 'OBS Creator') {
        throw 'The bundled preset must be named OBS Creator.'
    }
    $theme = $root.GetProperty('theme')
    $knownThemes = @('dark-classic', 'light-classic', 'dark-abstract', 'light-abstract',
        'manga-ink', 'steampunk-brass', 'cyberpunk-neon', 'moire', 'cubism', 'art-deco',
        'ukiyo-e', 'solarpunk', 'memphis')
    if ($theme.ValueKind -ne [System.Text.Json.JsonValueKind]::String -or
        $theme.GetString() -cnotin $knownThemes) {
        throw 'The existing settings must contain a supported theme.'
    }
    $setup = [System.Text.Json.JsonElement]::new()
    if ($root.TryGetProperty('setup_completed', [ref] $setup) -and
        $setup.ValueKind -notin @([System.Text.Json.JsonValueKind]::True, [System.Text.Json.JsonValueKind]::False)) {
        throw 'setup_completed must be a boolean when present.'
    }
    $pages = $root.GetProperty('pages')
    if ($pages.ValueKind -ne [System.Text.Json.JsonValueKind]::Array) {
        throw 'The existing settings must contain a pages array.'
    }
    $standardPages = @('Windows / media', 'Utility', 'Codex', 'VS Code', 'Spotify', 'Word', 'PowerPoint', 'Excel', 'Themes')
    $pageNames = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
    $rawPages = [System.Collections.Generic.Dictionary[string, string]]::new([System.StringComparer]::Ordinal)
    $customPages = [System.Collections.Generic.List[string]]::new()
    foreach ($page in $pages.EnumerateArray()) {
        Assert-Page $page
        $pageName = $page.GetProperty('name').GetString()
        if (-not $pageNames.Add($pageName)) {
            throw "Duplicate page name '$pageName'. Resolve it before installing this preset."
        }
        if ($pageName -ieq 'OBS Creator') {
            if (-not $Replace) {
                throw 'OBS Creator already exists. Use -Replace only to explicitly replace that page.'
            }
            continue
        }
        $rawPages.Add($pageName, $page.GetRawText())
        if ($pageName -cnotin $standardPages) { $customPages.Add($pageName) }
    }
    foreach ($name in $standardPages) {
        if (-not $rawPages.ContainsKey($name)) {
            throw "The existing settings are missing standard page '$name'. Use settings saved by the current application."
        }
    }
    $orderedPages = [System.Collections.Generic.List[string]]::new()
    foreach ($name in $standardPages[0..7]) { $orderedPages.Add($rawPages[$name]) }
    foreach ($name in $customPages) { $orderedPages.Add($rawPages[$name]) }
    $orderedPages.Add($presetDocument.RootElement.GetRawText())
    $orderedPages.Add($rawPages['Themes'])
    $newPages = "[`n" + [string]::Join(",`n", $orderedPages) + "`n]"
    $rootProperties = [System.Collections.Generic.List[string]]::new()
    foreach ($property in $root.EnumerateObject()) {
        # Preserve every existing value verbatim, including unknown extensions.
        $propertyName = ConvertTo-Json -InputObject $property.Name -Compress
        $rawValue = if ($property.Name -ceq 'pages') { $newPages } else { $property.Value.GetRawText() }
        $rootProperties.Add('  ' + $propertyName + ': ' + $rawValue)
    }
    $candidateText = "{`n" + [string]::Join(",`n", $rootProperties) + "`n}`n"
    $candidateDocument = [System.Text.Json.JsonDocument]::Parse($candidateText)

    $suffix = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffZ') + '-' + [Guid]::NewGuid().ToString('N')
    $temporaryPath = Join-Path $settingsItem.DirectoryName ('.obs-creator-' + $suffix + '.tmp')
    $backupPath = $targetPath + '.obs-creator-backup-' + $suffix + '.json'
    [System.IO.File]::WriteAllText($temporaryPath, $candidateText, [System.Text.UTF8Encoding]::new($false))
    if ([Convert]::ToBase64String([System.IO.File]::ReadAllBytes($targetPath)) -cne $originalFingerprint) {
        throw 'Settings changed while preparing the preset. Close UlanziDock and try again.'
    }
    # Same-directory atomic replacement also saves the exact previous file.
    # If the filesystem cannot support this operation, fail without a fallback.
    [System.IO.File]::Replace($temporaryPath, $targetPath, $backupPath)
    $temporaryPath = $null
    Write-Output "OBS Creator installed in: $targetPath"
    Write-Output "Original settings backup: $backupPath"
    Write-Output 'Configure the matching OBS hotkeys manually before using the buttons.'
} finally {
    if ($null -ne $candidateDocument) { $candidateDocument.Dispose() }
    if ($null -ne $presetDocument) { $presetDocument.Dispose() }
    if ($null -ne $settingsDocument) { $settingsDocument.Dispose() }
    if ($null -ne $temporaryPath -and [System.IO.File]::Exists($temporaryPath)) {
        [System.IO.File]::Delete($temporaryPath)
    }
}
