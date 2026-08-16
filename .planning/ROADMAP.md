# Roadmap: ReadToMe

## Overview

ReadToMe delivers one-keypress text-to-speech for any highlighted text, system-wide. The roadmap progresses from proving the riskiest piece (system-wide text capture via clipboard sandwich) through completing the core TTS loop, adding a settings UI, proving the pluggable provider architecture with a second provider, and finishing with advanced audio features. After Phase 2, the core value proposition works end-to-end.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation + Input Pipeline** - Tray app with global hotkey that captures selected text and persists configuration
- [ ] **Phase 2: Core TTS Loop** - Hotkey triggers text-to-speech via ElevenLabs with audio playback and toggle stop
- [ ] **Phase 3: Settings UI + Key Management** - Svelte settings window for provider configuration, API keys, and voice selection
- [ ] **Phase 4: Second Provider + Robustness** - Google Cloud TTS integration, long text chunking, and progress indication
- [ ] **Phase 5: Advanced Audio + Polish** - Streaming playback, speed control, device selection, and auto-start

## Phase Details

### Phase 1: Foundation + Input Pipeline
**Goal**: App runs silently in the system tray and reliably captures selected text from any application on demand
**Mode:** mvp
**Depends on**: Nothing (first phase)
**Requirements**: SYS-01, SYS-03, CAP-01, CAP-02, SET-01
**Success Criteria** (what must be TRUE):
  1. App launches to system tray with no main window or taskbar entry visible
  2. User can press a configured hotkey and see the currently selected text captured (verified via log output or tray notification)
  3. After text capture, the user's original clipboard contents are preserved (not overwritten)
  4. User's hotkey binding and provider settings survive app restart
  5. User can right-click the tray icon to access a context menu with at minimum a Quit option
**Plans**: TBD

### Phase 2: Core TTS Loop
**Goal**: User can hear any highlighted text read aloud with a single keypress and stop it with another
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-04, TTS-01, TTS-02, AUD-01
**Success Criteria** (what must be TRUE):
  1. User highlights text in any application, presses the hotkey, and hears it read aloud through their speakers
  2. User presses the hotkey again during playback and audio stops immediately
  3. Tray icon visually distinguishes between idle, processing, and reading states
  4. Rapid double-press of the hotkey does not accidentally cancel a reading that is still being processed (debounce protects the processing window)
  5. App handles errors gracefully (missing API key, network failure, empty selection) with a tray notification instead of crashing
**Plans**: TBD

### Phase 3: Settings UI + Key Management
**Goal**: User can configure all app settings through a graphical window without editing config files
**Mode:** mvp
**Depends on**: Phase 2
**Requirements**: SET-02, TTS-04, TTS-05
**Success Criteria** (what must be TRUE):
  1. User can open a settings window from the tray context menu and configure hotkey, provider, API key, and voice
  2. API keys are stored securely and displayed masked in the settings UI after initial entry
  3. User can browse and select from the list of voices available from their chosen TTS provider
  4. Settings changes take effect immediately without requiring an app restart
**Plans**: TBD
**UI hint**: yes

### Phase 4: Second Provider + Robustness
**Goal**: User can choose between TTS providers and read arbitrarily long text selections reliably
**Mode:** mvp
**Depends on**: Phase 3
**Requirements**: TTS-03, CAP-03, AUD-05
**Success Criteria** (what must be TRUE):
  1. User can select Google Cloud TTS as their provider and hear highlighted text read aloud
  2. Long text selections exceeding API character limits are automatically split at sentence boundaries and read as a continuous sequence
  3. User can see reading progress for long text via tray tooltip (e.g., "Reading chunk 3 of 7")
**Plans**: TBD

### Phase 5: Advanced Audio + Polish
**Goal**: User has full control over playback with low-latency streaming and system integration polish
**Mode:** mvp
**Depends on**: Phase 4
**Requirements**: AUD-02, AUD-03, AUD-04, SYS-02
**Success Criteria** (what must be TRUE):
  1. Audio begins playing before the full TTS synthesis response is received (streaming reduces perceived latency)
  2. User can adjust playback speed from the settings window
  3. User can select which audio output device to use for TTS playback
  4. User can enable auto-start so the app launches with the operating system
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation + Input Pipeline | 0/0 | Not started | - |
| 2. Core TTS Loop | 0/0 | Not started | - |
| 3. Settings UI + Key Management | 0/0 | Not started | - |
| 4. Second Provider + Robustness | 0/0 | Not started | - |
| 5. Advanced Audio + Polish | 0/0 | Not started | - |
