#!/usr/bin/env bash
# build-windows.sh — Trigger a Windows build from WSL2
#
# This copies the project to the Windows filesystem and runs the
# PowerShell build script there. Tauri's bundler tools (WiX, NSIS)
# cannot run from WSL UNC paths, so the copy is necessary.
#
# Usage:
#   ./scripts/build-windows.sh          # Release build
#   ./scripts/build-windows.sh --dev    # Dev mode (cargo tauri dev)
#   ./scripts/build-windows.sh --debug  # Debug build (faster, unoptimized)
#
# Prerequisites:
#   - Rust installed on Windows (not WSL): https://rustup.rs
#   - Visual Studio Build Tools with C++ workload
#   - Node.js 18+ on Windows

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Windows user home — override by exporting WIN_USER if auto-detection is wrong.
# Detection runs under `set -euo pipefail`, where a failed command substitution
# aborts the script outright; with powershell's stderr sent to /dev/null that
# abort is completely silent. Guard the call and validate the result instead.
WIN_USER="${WIN_USER:-}"
if [ -z "$WIN_USER" ] && command -v powershell.exe >/dev/null 2>&1; then
    WIN_USER=$(powershell.exe -NoProfile -Command '[Environment]::UserName' 2>/dev/null | tr -d '\r') \
        || WIN_USER=""
fi

if [ -z "$WIN_USER" ]; then
    echo "ERROR: Could not determine your Windows username." >&2
    echo "This script must run from WSL2 with Windows interop enabled, so that" >&2
    echo "powershell.exe is callable from Linux." >&2
    echo "" >&2
    echo "To bypass detection, set it explicitly:" >&2
    echo "    WIN_USER=yourname $0 $*" >&2
    exit 1
fi

WIN_BUILD_DIR="/mnt/c/Users/${WIN_USER}/readtome-build"
WIN_BUILD_PATH="C:\\Users\\${WIN_USER}\\readtome-build"

MODE=""
PS_FLAG=""
case "${1:-}" in
    --dev)   MODE="dev";   PS_FLAG="-Dev"   ;;
    --debug) MODE="debug"; PS_FLAG="-Debug" ;;
    *)       MODE="release"; PS_FLAG=""      ;;
esac

echo "=== ReadToMe Windows Build (${MODE}) ==="
echo "Source:  ${PROJECT_DIR}"
echo "Target:  ${WIN_BUILD_DIR}"
echo ""

# Sync project to Windows filesystem (exclude build artifacts)
echo "Syncing project to Windows filesystem..."
rsync -a --delete \
    --exclude 'node_modules/' \
    --exclude 'target/' \
    --exclude 'build/' \
    --exclude 'dist/' \
    --exclude '.gsd/gsd.db*' \
    --exclude '.gsd/runtime/' \
    --exclude '.gsd/exec/' \
    --exclude '.bg-shell/' \
    "${PROJECT_DIR}/" "${WIN_BUILD_DIR}/"

echo "Sync complete."
echo ""

# Run the PowerShell build script on the Windows side
echo "Starting Windows build..."
# `set -e` would abort the script before we could inspect $?, so capture the
# status inline via `||` — the left side of `||` is exempt from errexit.
BUILD_EXIT=0
powershell.exe -NoProfile -ExecutionPolicy Bypass \
    -File "${WIN_BUILD_PATH}\\scripts\\build-windows.ps1" ${PS_FLAG} || BUILD_EXIT=$?

if [ "$BUILD_EXIT" -ne 0 ]; then
    echo "" >&2
    echo "Windows build failed (exit ${BUILD_EXIT})." >&2
    exit "$BUILD_EXIT"
fi

if [ "$MODE" = "release" ]; then
    echo ""
    echo "=== Copying installers back to WSL ==="
    mkdir -p "${PROJECT_DIR}/dist"

    BUNDLE_DIR="${WIN_BUILD_DIR}/src-tauri/target/release/bundle"

    # Copy NSIS installer
    if ls "${BUNDLE_DIR}/nsis/"*.exe 1>/dev/null 2>&1; then
        cp "${BUNDLE_DIR}/nsis/"*.exe "${PROJECT_DIR}/dist/"
        echo "NSIS installer copied to dist/"
    fi

    # Copy MSI installer
    if ls "${BUNDLE_DIR}/msi/"*.msi 1>/dev/null 2>&1; then
        cp "${BUNDLE_DIR}/msi/"*.msi "${PROJECT_DIR}/dist/"
        echo "MSI installer copied to dist/"
    fi

    # Copy standalone exe
    EXE="${WIN_BUILD_DIR}/src-tauri/target/release/ReadToMe.exe"
    if [ -f "$EXE" ]; then
        cp "$EXE" "${PROJECT_DIR}/dist/"
        echo "Standalone exe copied to dist/"
    fi

    echo ""
    echo "Build artifacts in ${PROJECT_DIR}/dist/:"
    ls -lh "${PROJECT_DIR}/dist/" 2>/dev/null
fi

exit 0
