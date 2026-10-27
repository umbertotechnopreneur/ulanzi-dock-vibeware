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
Forward custom icon-output paths to the shared MSIX artwork generator.
.DESCRIPTION
This thin entry point only requests icon generation. It does not build, sign, install
or publish an MSIX. With no parameters it opens the tool menu.
.PARAMETER DetailedPath
Optional destination for the detailed installer-icon master.
.PARAMETER CompactPath
Optional destination for the small-size installer-icon master.
#>

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
