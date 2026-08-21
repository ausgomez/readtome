---
quick_id: 260821-fcm
type: quick
subsystem: build-tooling
tags: [bash, errexit, pipefail, wsl, tauri, windows-build]
provides:
  - Explicit exit-status handling for the WSL -> Windows build bridge
  - Validated Windows-username detection with an env override
affects: [windows-build, release-packaging]
actuals:
  tasks: 2
  commits: 0
tech-stack:
  added: []
  patterns:
    - "errexit-safe status capture via `|| VAR=$?`"
    - "validate-then-fail-loud on auto-detected values"
key-files:
  created: []
  modified: [scripts/build-windows.sh]
key-decisions:
  - "Fail fast with a diagnostic instead of silently falling through"
  - "WIN_USER honoured from the environment, making the error message actionable"
duration: ~15min
completed: 2026-08-21
status: complete
---

# Quick Task 260821-fcm Summary

**`scripts/build-windows.sh` had two places where `set -euo pipefail` silently
defeated the script's own error handling; both now report and exit deliberately.**

## Performance
- **Duration:** ~15 min
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

**1. Dead `BUILD_EXIT` after the PowerShell invocation**
- Replaced the post-call `BUILD_EXIT=$?` (unreachable under `errexit`) with
  `BUILD_EXIT=0` + `|| BUILD_EXIT=$?`.
- Added an explicit failure branch printing `Windows build failed (exit N).`
  to stderr. Previously a failed build aborted the wrapper with no message.
- Copy guard reduced to `[ "$MODE" = "release" ]`; trailing
  `exit $BUILD_EXIT` reduced to `exit 0`, both now reachable only on success.

**2. Silent abort in `WIN_USER` detection**
- Detection is now gated behind `command -v powershell.exe` and suffixed with
  `|| WIN_USER=""`, so neither absence nor failure aborts the script.
- Result is validated non-empty, with an actionable error naming the override.
- `WIN_USER` is honoured from the environment, skipping detection entirely.

## Files Created/Modified
- `scripts/build-windows.sh` - errexit-safe capture of the PowerShell build
  status; guarded and validated Windows-username detection.

## Verification

`bash -n scripts/build-windows.sh` - syntax OK. shellcheck not installed on
this host, so it was not run.

Differential harnesses (old structure vs new, with stubs):

| Scenario | Old behaviour | New behaviour |
|---|---|---|
| Build returns 7 | capture line unreachable, no message, exit 7 | `Windows build failed (exit 7).`, exit 7 |
| Build succeeds | - | reaches copy section, exit 0 |
| `powershell.exe` absent | exit 127, **zero output** | actionable error, exit 1 |
| `powershell.exe` returns empty, status 0 | `/mnt/c/Users//readtome-build` used as an `rsync --delete` target | actionable error, exit 1 |
| `powershell.exe` works | `WIN_USER` set | unchanged |
| `WIN_USER` preset in env | ignored, detection ran anyway | detection skipped, value used |

Not exercised end-to-end against a real Windows build - that needs WSL2 plus a
Windows-side Rust/Node toolchain, unavailable on this macOS host.

## Notes / Follow-ups
- None outstanding. The `WIN_USER` env override is worth a line in the script's
  usage comment block or README if the WSL path gets documented further.
