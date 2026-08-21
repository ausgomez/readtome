---
quick_id: 260821-fcm
type: quick
created: 2026-08-21
---

# Quick Task 260821-fcm: errexit-safe status handling in build-windows.sh

Two defects of the same class in `scripts/build-windows.sh`, where `set -euo
pipefail` silently defeats the script's own error handling.

## Problem 1 — dead `BUILD_EXIT`

`scripts/build-windows.sh` runs `set -euo pipefail` at the top, then invokes the
Windows PowerShell build and attempts to capture its status:

```bash
powershell.exe ... -File "...\build-windows.ps1" ${PS_FLAG}

BUILD_EXIT=$?
```

Under `errexit`, a non-zero return from `powershell.exe` aborts the script
immediately — control never reaches `BUILD_EXIT=$?`. The variable is therefore
always `0` wherever it is read, making both `[ $BUILD_EXIT -eq 0 ]` and the
trailing `exit $BUILD_EXIT` dead logic. Failures still propagated, but only as a
side effect of errexit, with no diagnostic.

### Fix

Capture the status inline via `||` (the left operand of `||` is exempt from
errexit), then branch on it explicitly:

- Initialise `BUILD_EXIT=0` before the call.
- Append `|| BUILD_EXIT=$?` to the `powershell.exe` invocation.
- Add an explicit failure branch that prints a diagnostic to stderr and exits
  with the captured status.
- Reduce the artifact-copy guard to `[ "$MODE" = "release" ]`, since a failing
  build now exits before reaching it.
- Change the trailing `exit $BUILD_EXIT` to `exit 0` — it is only reachable on
  success.

## Problem 2 — silent abort in `WIN_USER` detection

```bash
WIN_USER=$(powershell.exe -NoProfile -Command '[Environment]::UserName' 2>/dev/null | tr -d '\r')
WIN_BUILD_DIR="/mnt/c/Users/${WIN_USER}/readtome-build"
```

Two failure modes, both silent:

1. `powershell.exe` unreachable (script run outside WSL, or interop disabled):
   the command substitution fails, `errexit` aborts the script, and because
   stderr is routed to `/dev/null` the user sees *no output at all* — just a
   bare exit 127.
2. `powershell.exe` present but returning an empty string with status 0:
   detection "succeeds" with `WIN_USER=""`, producing the path
   `/mnt/c/Users//readtome-build`. That path is subsequently used as an
   `rsync -a --delete` destination, so a wrong-but-plausible directory gets
   synced and pruned.

### Fix

- Honour a pre-set `WIN_USER` environment variable, skipping detection
  entirely (this also gives the existing "adjust if your username differs"
  comment a real mechanism).
- Gate the call behind `command -v powershell.exe` and append `|| WIN_USER=""`
  so neither absence nor failure can abort the script.
- Validate the result is non-empty and exit 1 with an actionable message
  naming the `WIN_USER=` override, covering both failure modes above.

## Scope

- `scripts/build-windows.sh` only. No behaviour change to
  `scripts/build-windows.ps1`, CI, or the Tauri bundle config.
- The `WIN_USER` environment override is a small addition beyond a pure bug
  fix; it is what makes the new error message actionable.

## Verification

- `bash -n scripts/build-windows.sh` parses clean.
- `BUILD_EXIT`: harness reproducing old vs new structure with a stub returning
  7 — old form never reaches the capture line; new form prints the diagnostic
  and exits 7; success path still reaches the copy section and exits 0.
- `WIN_USER`: harness over four scenarios with a stub `powershell.exe` —
  absent, failing, working, and env-override. Old form exits 127 with no
  output (absent) and yields `/mnt/c/Users//readtome-build` (empty result);
  new form reports and exits 1 in both, and passes through cleanly in the
  working and override cases.
