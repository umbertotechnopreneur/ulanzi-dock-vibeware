#!/usr/bin/env bash
# VBWR B
#
# Project: UlanziDeck
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
#
# VibeWare: Human intent, AI execution, and plenty of tokens
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
#
# Modified with AI: OpenAI Codex; explicit offline Linux build script; 2026-11-02
# Human guidance: Umberto Giacobbi; isolated Linux migration worktree
# Evidence: current source
#
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
#
# VBWR E

# Prepare and scan the toolchain and cached dependencies before invoking this script.
# CARGO_HOME, CARGO_TARGET_DIR and TMPDIR are inherited from the caller.
# This script builds release binaries only. It does not run them or clean the cache.
set -euo pipefail

if (( $# != 0 )); then
    printf '%s\n' 'Usage: bash scripts/build-linux.sh' >&2
    exit 2
fi

if [[ "$OSTYPE" != linux* ]]; then
    printf '%s\n' 'Run this build script on Linux.' >&2
    exit 2
fi

if ! command -v cargo >/dev/null 2>&1; then
    printf '%s\n' \
        'Cargo was not found. Prepare Rust 1.90 or newer and scan the toolchain and dependencies before building.' >&2
    exit 127
fi

project_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd -- "$project_dir"

# Also prevent a Cargo executable managed by rustup from installing a toolchain.
export RUSTUP_AUTO_INSTALL=0
exec cargo build --frozen --release --bins
