# ReadToMe

## What This Is

A system tray application that reads highlighted text aloud using text-to-speech. The user presses a global hotkey, and whatever text is currently selected anywhere on their system gets read out in a natural-sounding voice. Press the hotkey again to stop. That's the core interaction — invisible until needed, then immediately useful.

## Why It Exists

Some people process information better by listening than reading. Dense text walls — especially AI-generated output from tools like Claude or Codex — can be hard to parse visually. Having someone read it aloud helps comprehension. ReadToMe replaces "someone" with a high-quality TTS voice, available on demand with a single keypress.

## Who It's For

Anyone who benefits from hearing text read aloud:
- Users with reading difficulties or visual fatigue
- People working with dense AI-generated output
- Anyone who processes spoken information more easily than written text

## Core Value

**One-keypress text-to-speech for any highlighted text, system-wide.** If this doesn't work reliably and instantly, nothing else matters.

## How It Works

1. App runs in the system tray — no visible window during normal use
2. User highlights text in any application
3. User presses the configured hotkey
4. App captures the selected text (via clipboard or OS selection API)
5. Text is sent to the configured TTS provider
6. Audio plays back through the system's default audio output
7. User presses the hotkey again to stop playback

## Technical Approach

- **Framework:** Tauri (Rust backend, web frontend) — cross-platform from one codebase, small footprint (~5-10MB)
- **TTS:** Pluggable provider architecture. Ships with ElevenLabs and Google TTS integrations. Users can choose their preferred provider in settings.
- **Platform:** Windows first, macOS second. Tauri handles both from the same codebase.
- **UI:** System tray icon + settings window (voice selection, hotkey configuration, TTS provider setup)

## Context

- **Text capture:** Global hotkey intercept + clipboard/selection reading. Must work across all applications — browsers, editors, PDFs, desktop apps.
- **Audio:** Streams or plays synthesized speech through system default output device.
- **Settings:** Minimal — hotkey binding, TTS provider selection, API key entry, voice selection, possibly playback speed.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] System tray app that starts minimized and runs in background
- [ ] Global hotkey registration (user-configurable)
- [ ] Capture currently selected/highlighted text from any application
- [ ] Send captured text to TTS provider and play audio
- [ ] Toggle behavior: hotkey starts reading, same hotkey stops
- [ ] ElevenLabs TTS provider integration
- [ ] Google TTS provider integration
- [ ] Pluggable TTS provider architecture (easy to add new providers)
- [ ] Settings window: hotkey configuration, provider selection, API key entry, voice selection
- [ ] Windows support
- [ ] macOS support

### Out of Scope

- Pause/resume or speed controls (v1 is toggle-only: play or stop)
- Reading history or text display window
- Sentence-by-sentence stepping
- Offline/local TTS engines (cloud-only for v1)
- Browser extension or editor plugin approach (this is a native system-wide tool)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri over Electron | Much smaller binary (~5-10MB vs ~150MB), sufficient for tray app + settings UI, single codebase for Windows + macOS | — Pending |
| Tauri over native-per-platform | Halves the codebase; tray app has minimal UI surface so deep OS integration isn't critical | — Pending |
| Cloud TTS only for v1 | ElevenLabs and Google TTS provide natural-sounding voices; local engines can be added later as a provider | — Pending |
| Toggle interaction model | Simplest possible UX — one hotkey does everything. No cognitive overhead. | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-08-15 after initialization*
