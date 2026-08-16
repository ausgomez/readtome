# build-windows.ps1 — Build ReadToMe for Windows
# Run from PowerShell on Windows (not WSL).
#
# Usage:
#   .\scripts\build-windows.ps1          # Release build (NSIS + MSI installers)
#   .\scripts\build-windows.ps1 -Dev     # Dev build (just the .exe, faster)
#   .\scripts\build-windows.ps1 -Debug   # Debug build (unoptimized, with symbols)
#
# Prerequisites (one-time):
#   1. Install Rust: https://rustup.rs (use default MSVC toolchain)
#   2. Install Visual Studio Build Tools with "Desktop development with C++"
#   3. Install Node.js 18+: https://nodejs.org
#   4. WebView2 Runtime (pre-installed on Windows 10 21H2+ / Windows 11)

param(
    [switch]$Dev,
    [switch]$Debug
)

$ErrorActionPreference = "Stop"

# Ensure we're in the project root (the parent of this script's directory).
# When invoked from WSL, the working directory is a UNC path that Windows
# doesn't support, so it falls back to C:\Windows — breaking npm/cargo.
Set-Location (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))

Write-Host "ReadToMe Windows Build" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Cyan

# Check prerequisites
$missing = @()

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    $missing += "Rust (install from https://rustup.rs)"
}
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    $missing += "Node.js (install from https://nodejs.org)"
}
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    $missing += "npm (comes with Node.js)"
}

if ($missing.Count -gt 0) {
    Write-Host "`nMissing prerequisites:" -ForegroundColor Red
    foreach ($m in $missing) {
        Write-Host "  - $m" -ForegroundColor Yellow
    }
    Write-Host "`nInstall these first, then re-run this script." -ForegroundColor Red
    exit 1
}

Write-Host "Rust:    $(rustc --version)" -ForegroundColor Gray
Write-Host "Node:    $(node --version)" -ForegroundColor Gray
Write-Host "npm:     $(npm --version)" -ForegroundColor Gray

# Install npm dependencies
Write-Host "`nInstalling npm dependencies..." -ForegroundColor Cyan
npm install
if ($LASTEXITCODE -ne 0) { throw "npm install failed" }

if ($Dev) {
    # Dev mode — launch with hot reload
    Write-Host "`nStarting dev server..." -ForegroundColor Green
    npx tauri dev
} elseif ($Debug) {
    # Debug build — unoptimized .exe only
    Write-Host "`nBuilding debug..." -ForegroundColor Cyan
    npx tauri build --debug
    if ($LASTEXITCODE -ne 0) { throw "Build failed" }

    $exe = "src-tauri\target\debug\ReadToMe.exe"
    if (Test-Path $exe) {
        Write-Host "`nDebug build complete:" -ForegroundColor Green
        Write-Host "  $exe" -ForegroundColor White
    }
} else {
    # Release build — optimized .exe + installers
    Write-Host "`nBuilding release..." -ForegroundColor Cyan
    npx tauri build
    if ($LASTEXITCODE -ne 0) { throw "Build failed" }

    $bundleDir = "src-tauri\target\release\bundle"
    Write-Host "`nRelease build complete! Installers:" -ForegroundColor Green

    # NSIS installer
    $nsis = Get-ChildItem "$bundleDir\nsis\*.exe" -ErrorAction SilentlyContinue
    foreach ($f in $nsis) {
        Write-Host "  NSIS:  $($f.FullName)" -ForegroundColor White
        Write-Host "         $([math]::Round($f.Length / 1MB, 1)) MB" -ForegroundColor Gray
    }

    # MSI installer
    $msi = Get-ChildItem "$bundleDir\msi\*.msi" -ErrorAction SilentlyContinue
    foreach ($f in $msi) {
        Write-Host "  MSI:   $($f.FullName)" -ForegroundColor White
        Write-Host "         $([math]::Round($f.Length / 1MB, 1)) MB" -ForegroundColor Gray
    }

    # Standalone exe
    $exe = "src-tauri\target\release\ReadToMe.exe"
    if (Test-Path $exe) {
        $size = [math]::Round((Get-Item $exe).Length / 1MB, 1)
        Write-Host "  EXE:   $((Resolve-Path $exe).Path)" -ForegroundColor White
        Write-Host "         $size MB (standalone, no install needed)" -ForegroundColor Gray
    }
}
