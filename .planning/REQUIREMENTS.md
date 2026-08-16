# ReadToMe — v1 Requirements

## v1 Requirements

### Core Interaction

- [ ] **CORE-01**: User can press a global hotkey and hear the currently highlighted text read aloud
- [ ] **CORE-02**: User can press the same hotkey again to stop playback immediately
- [ ] **CORE-03**: App shows visual feedback via tray icon state change when reading vs idle
- [ ] **CORE-04**: App debounces hotkey presses to prevent accidental stop during TTS processing delay

### System Integration

- [ ] **SYS-01**: App runs as a system tray icon with no main window visible during normal use
- [ ] **SYS-02**: App auto-starts with the operating system (opt-in)
- [ ] **SYS-03**: User can configure the trigger hotkey to avoid conflicts with other apps

### Text Capture

- [ ] **CAP-01**: App captures currently selected/highlighted text from any application system-wide
- [ ] **CAP-02**: App preserves clipboard contents by saving before capture and restoring after
- [ ] **CAP-03**: App splits long text selections into chunks at sentence boundaries to respect API character limits

### TTS Providers

- [ ] **TTS-01**: App supports a pluggable TTS provider architecture (trait-based abstraction)
- [ ] **TTS-02**: User can send text to ElevenLabs API and hear the result
- [ ] **TTS-03**: User can send text to Google Cloud TTS API and hear the result
- [ ] **TTS-04**: User can securely store and manage API keys per provider
- [ ] **TTS-05**: User can select a voice from the provider's available voice list

### Audio

- [ ] **AUD-01**: App plays synthesized speech through the system's default audio output device
- [ ] **AUD-02**: User can adjust playback speed
- [ ] **AUD-03**: App streams audio playback (starts playing before full synthesis completes)
- [ ] **AUD-04**: User can select which audio output device to use
- [ ] **AUD-05**: App shows reading progress for long text (e.g., tray tooltip "3/7 chunks")

### Settings

- [ ] **SET-01**: User settings persist across app restarts
- [ ] **SET-02**: User can configure hotkey, provider, API keys, voice, and speed in a settings window

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| CORE-01 | Phase 2 | Pending |
| CORE-02 | Phase 2 | Pending |
| CORE-03 | Phase 2 | Pending |
| CORE-04 | Phase 2 | Pending |
| SYS-01 | Phase 1 | Pending |
| SYS-02 | Phase 5 | Pending |
| SYS-03 | Phase 1 | Pending |
| CAP-01 | Phase 1 | Pending |
| CAP-02 | Phase 1 | Pending |
| CAP-03 | Phase 4 | Pending |
| TTS-01 | Phase 2 | Pending |
| TTS-02 | Phase 2 | Pending |
| TTS-03 | Phase 4 | Pending |
| TTS-04 | Phase 3 | Pending |
| TTS-05 | Phase 3 | Pending |
| AUD-01 | Phase 2 | Pending |
| AUD-02 | Phase 5 | Pending |
| AUD-03 | Phase 5 | Pending |
| AUD-04 | Phase 5 | Pending |
| AUD-05 | Phase 4 | Pending |
| SET-01 | Phase 1 | Pending |
| SET-02 | Phase 3 | Pending |

## v2 Requirements (Deferred)

- Pause/resume playback
- Multi-language auto-detection with voice switching
- Clipboard monitoring mode (auto-read on copy)
- Pronunciation customization (custom dictionary)

## Out of Scope

- Document reader / file import — different product (NaturalReader, Speechify)
- Word-by-word text highlighting — requires visible window, contradicts tray-only design
- OCR / screenshot-to-text — separate technology stack, users can select text already
- Browser extension — global hotkey already covers browsers
- Offline/local TTS — cloud-first for v1; pluggable arch allows later addition
- Reading history — utility should be stateless for privacy
- Voice cloning — premium feature orthogonal to core use case
- Dictation / speech-to-text — wrong direction; app reads TO you
- AI grammar features — feature creep, dilutes core value
- Cross-device sync — desktop utility, no reading position to sync
- Subscription system — users bring their own API keys

---
*Last updated: 2026-08-15 after roadmap creation*
