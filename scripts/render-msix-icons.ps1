# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# Kept as a small entry point for callers using custom icon paths.
[CmdletBinding()]
param(
    [string] $DetailedPath,
    [string] $CompactPath
)

if ($PSBoundParameters.Count -eq 0) {
    & (Join-Path $PSScriptRoot 'ulanzi.ps1')
    return
}
& (Join-Path $PSScriptRoot 'build-msix.ps1') -IconsOnly @PSBoundParameters
