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

# Windows user home — adjust if your Windows username differs
WIN_USER=$(powershell.exe -NoProfile -Command '[Environment]::UserName' 2>/dev/null | tr -d '\r')
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
powershell.exe -NoProfile -ExecutionPolicy Bypass \
    -File "${WIN_BUILD_PATH}\\scripts\\build-windows.ps1" ${PS_FLAG}

BUILD_EXIT=$?

if [ $BUILD_EXIT -eq 0 ] && [ "$MODE" = "release" ]; then
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

exit $BUILD_EXIT
