# Project Research Summary

**Project:** ReadToMe
**Domain:** Desktop system tray TTS utility (Tauri v2 + cloud TTS)
**Researched:** 2026-08-15
**Confidence:** MEDIUM

## Executive Summary

ReadToMe is a system tray utility that reads highlighted text aloud via cloud TTS -- a well-understood product category with clear precedents (TinyReadAloud, NaturalReader, Vox). The recommended approach uses Tauri v2 with a Rust backend handling all critical operations (hotkey listening, clipboard manipulation, TTS API calls, audio playback via rodio) and a minimal Svelte 5 frontend solely for settings. This architecture avoids the most dangerous pitfall class: webview-dependent audio that breaks in production builds.

The core technical risk is the text capture pipeline. Capturing selected text system-wide requires simulating Ctrl+C/Cmd+C via the `enigo` crate, reading the clipboard via `arboard`, and restoring original clipboard contents -- a "clipboard sandwich" pattern with timing-sensitive race conditions. On macOS, this entire flow silently fails without Accessibility permissions, which the OS does not auto-prompt for. These two issues (clipboard clobbering and macOS permissions) must be solved in the earliest phases or the app is fundamentally broken.

The second major risk is perceived latency. The chain from hotkey press to audio output spans 500ms-2000ms+ (clipboard simulation + network RTT + TTS synthesis + audio buffer fill). Without immediate visual feedback and eventual streaming playback, users will double-press the hotkey (triggering stop via toggle logic) and conclude the app is broken. The architecture should be designed for streaming from the start, even if the initial implementation waits for full audio -- retrofitting streaming is significantly harder than building the plumbing upfront.

## Key Findings

### Recommended Stack

Tauri v2 (2.11.x) with Rust backend and Svelte 5 frontend. The frontend is intentionally minimal -- a single settings window with ~5 form inputs. Svelte was chosen over React for smallest bundle size; React's ecosystem advantages are irrelevant at this UI scope.

**Core technologies:**
- **Tauri v2:** App shell, system tray, global hotkey, IPC -- already decided, mature since Oct 2024
- **Svelte 5 + Vite 6 + TypeScript:** Settings UI only -- smallest bundle, official Tauri template
- **rodio 0.21:** Rust-side audio playback -- avoids all webview audio pitfalls, works without visible window
- **reqwest 0.13 (async + streaming):** HTTP client for TTS APIs -- supports chunked streaming for ElevenLabs
- **enigo 0.3 + arboard 3:** Text capture via clipboard sandwich pattern -- simulates copy, reads clipboard, restores
- **tauri-plugin-global-shortcut, store, clipboard-manager, notification:** Official Tauri plugins for OS integration

**Critical note:** No Rust SDKs exist for ElevenLabs or Google Cloud TTS. All provider calls are direct REST via reqwest. ElevenLabs `eleven_flash_v2_5` model recommended as default (75ms latency).

### Expected Features

**Must have (table stakes):**
- Global hotkey to read selected text (the entire product premise)
- System tray presence with no main window
- Toggle stop (press again to stop)
- Voice selection and playback speed control
- Settings persistence across restarts
- API key management (secure)
- Configurable hotkey (conflicts are common)
- Visual feedback during reading (tray icon state change minimum)
- Multiple TTS provider support (ElevenLabs + Google)

**Should have (differentiators):**
- Streaming audio playback (low latency) -- single biggest UX differentiator
- Intelligent text chunking for long selections (API limits: ~5000 chars)
- Clipboard-preserving text capture (save/restore)
- Reading progress indicator for long text

**Defer (v2+):**
- Pause/resume (PROJECT.md explicitly defers)
- Audio output device selection
- Multi-language auto-detection
- Clipboard monitoring mode (auto-read on copy)
- Pronunciation customization
- Auto-start with OS

**Anti-features (never build):**
- Document reader / file import, word highlighting, OCR, built-in editor, browser extension, offline TTS (v1), reading history, voice cloning, dictation, AI grammar features, cross-device sync, subscription system

### Architecture Approach

Layered Tauri architecture with Rust backend owning all critical paths. The hotkey listener, text capture, TTS calls, and audio playback all run in Rust -- the webview is only used for the settings UI. Key pattern: trait-based TTS provider abstraction (`TtsProvider` trait with `synthesize` and `list_voices` methods) enabling pluggable backends. State managed via `tauri::State<Mutex<T>>` for thread-safe access across async command handlers.

**Major components:**
1. **Tray Manager** (`tray.rs`) -- system tray icon, context menu, tooltip, app lifecycle
2. **Hotkey Listener** (`hotkey.rs`) -- global shortcut registration, toggle state machine (idle/reading)
3. **Text Capture** (`capture.rs`) -- clipboard sandwich: save, simulate Ctrl+C, read, restore
4. **TTS Provider Trait + Implementations** (`tts/`) -- provider abstraction, ElevenLabs and Google backends
5. **Audio Playback** (`audio.rs`) -- rodio sink-based playback with stop control via handle
6. **Config/State** (`config.rs`) -- settings persistence, managed Tauri state
7. **Settings UI** (`src/`) -- Svelte frontend for configuration via Tauri IPC commands

### Critical Pitfalls

1. **Clipboard clobbering** -- Save/restore clipboard around every text capture. Add 50-100ms delay for OS processing. Handle multiple clipboard formats, not just text. Must be correct from the first text capture implementation.
2. **macOS Accessibility permissions** -- Global hotkey and input simulation silently fail without permissions. Must detect on first launch and show onboarding dialog with direct link to System Settings. No workaround exists.
3. **Perceived latency (500ms-2s+ gap)** -- Show immediate visual feedback on hotkey press. Design for streaming architecture upfront. Use Flash/Turbo TTS models. Add debounce to prevent double-press cancellation during processing window.
4. **API keys in frontend** -- Store keys exclusively in Rust backend. Use OS keychain via `keyring` crate. Never return actual key to JS layer. Mask in settings UI after entry.
5. **Audio works in dev, breaks in production** -- Use Rust-side rodio, not webview audio. This is an architectural decision that eliminates the entire class of CSP/autoplay/format webview issues.

## Implications for Roadmap

### Phase 1: Foundation (Tray + Config)
**Rationale:** Everything depends on the app running as a tray application with persistent config. This is the skeleton.
**Delivers:** App that launches to system tray, shows context menu, persists settings, quits cleanly. No visible window in taskbar/dock.
**Addresses:** System tray presence, settings persistence
**Avoids:** Tray icon visibility issues (#8) -- provide template icons, test light/dark themes. Version pinning (#12) -- lock all deps from first commit. macOS dock icon (#8) -- use ActivationPolicy::Accessory.

### Phase 2: Input Pipeline (Hotkey + Text Capture)
**Rationale:** This is the highest-risk phase. If clipboard simulation or hotkey registration fails on a platform, everything downstream is worthless. Must be validated early.
**Delivers:** Hotkey press captures selected text from any application, with clipboard preservation. Console logging proves capture works.
**Addresses:** Global hotkey registration, text capture, configurable hotkey, clipboard preservation
**Avoids:** Clipboard clobbering (#1), macOS permissions (#2), hotkey conflicts (#7), text capture failures in terminals (#9)

### Phase 3: TTS Integration (Provider Trait + ElevenLabs)
**Rationale:** With text capture proven, connect to the TTS API. ElevenLabs first because streaming support is superior and voice quality is best-in-class. Build the provider abstraction trait but only implement one backend.
**Delivers:** Captured text sent to ElevenLabs API, audio bytes received. Provider trait designed for extensibility.
**Addresses:** ElevenLabs integration, pluggable provider architecture, text chunking for long selections
**Avoids:** Character limit silent failures (#6), latency gap (#3) -- use streaming endpoint and Flash model from the start. Connection management (#10) -- use HTTP streaming, not WebSockets initially.

### Phase 4: Audio Playback + Toggle
**Rationale:** Completes the core loop. After this phase, the product delivers its core value: hotkey -> capture -> TTS -> playback -> stop.
**Delivers:** Full end-to-end flow working. Audio plays through system output via rodio. Second hotkey press stops playback. Tray icon reflects state (idle/reading/error).
**Addresses:** Audio playback, toggle stop, visual feedback
**Avoids:** Audio production build failures (#5) -- rodio in Rust eliminates webview issues. Synchronous TTS blocking (anti-pattern 2) -- spawn async, store handle.

### Phase 5: Settings UI
**Rationale:** With core flow working, make it user-configurable. Until now, config can be hardcoded or file-edited.
**Delivers:** Settings window accessible from tray menu. API key entry (masked, stored securely), voice picker (populated from provider API), hotkey rebinding, provider selection, speed control.
**Addresses:** API key management, voice selection, playback speed, configurable hotkey UI
**Avoids:** API keys in frontend (#4) -- keys go to Rust backend only, stored in OS keychain. Silent misconfiguration (#13) -- validate API key on entry with test call.

### Phase 6: Second Provider + Polish
**Rationale:** Adds Google TTS as second provider, proving the abstraction works. Polish error handling and edge cases.
**Delivers:** Google TTS provider, comprehensive error notifications, API key validation, usage warnings, auto-start option.
**Addresses:** Google TTS integration, error feedback, auto-start
**Avoids:** Google auth complexity (#11) -- accept API key (simpler) rather than service account JSON for desktop use.

### Phase Ordering Rationale

- **Risk-first:** Phase 2 (input pipeline) is the riskiest -- OS-level clipboard simulation and hotkey registration have platform-specific failure modes. Proving this works before investing in TTS and audio code avoids wasted effort.
- **Dependency-driven:** Each phase produces a testable increment that the next phase builds on. Config -> Input -> TTS -> Audio -> UI -> Polish.
- **Architecture grouping:** TTS provider trait (Phase 3) and its second implementation (Phase 6) are separated so the abstraction is designed under real constraints but doesn't delay the core loop.
- **Pitfall alignment:** The most critical pitfalls (clipboard, macOS permissions, latency) are addressed in Phases 2-4 where they naturally occur.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2:** Text capture timing, enigo platform quirks, macOS Accessibility permission detection API -- needs hands-on spike
- **Phase 3:** ElevenLabs streaming response format, chunked audio decoding with rodio, text chunking strategy at sentence boundaries
- **Phase 4:** rodio streaming source implementation for progressive audio playback, sink lifecycle management

Phases with standard patterns (skip research-phase):
- **Phase 1:** Standard Tauri tray app setup, well-documented in official guides
- **Phase 5:** Standard Tauri IPC commands + Svelte forms, established patterns
- **Phase 6:** Same provider trait implementation as Phase 3, just different API shape

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM | All crate versions verified against crates.io. Tauri v2 is stable. enigo confidence is LOW (platform quirks not fully documented). |
| Features | MEDIUM | Cross-verified across 8+ competitor tools. Feature landscape is well-understood for this product category. |
| Architecture | MEDIUM | Layered Tauri pattern is standard. Clipboard sandwich pattern is proven by similar tools (Bob translator, OpenAI translator). |
| Pitfalls | MEDIUM | All pitfalls corroborated by Tauri GitHub issues and multiple developer sources. No single-source findings. |

**Overall confidence:** MEDIUM

### Gaps to Address

- **enigo reliability:** LOW confidence on cross-platform keyboard simulation. The crate is well-maintained but platform edge cases (UWP apps on Windows, sandboxed apps on macOS) are not well-documented. Plan a hands-on spike in Phase 2.
- **Streaming audio with rodio:** No clear examples of feeding chunked HTTP response data progressively into a rodio sink. May need a custom `Source` implementation or intermediate buffer. Research needed in Phase 3-4 planning.
- **Playback speed control:** ElevenLabs does not support speed control in the API. Must be done client-side via audio resampling. rodio supports this via `Source::speed()` but quality at extreme rates is unknown.
- **macOS Accessibility permission detection:** Exact Rust API for checking Accessibility permission status needs investigation. May need a small Swift/ObjC bridge or the `macos-accessibility-client` crate.
- **Secure key storage:** `keyring` crate is recommended but not yet validated for this use case. Fallback to encrypted config file may be needed on Linux.

## Sources

### Primary (direct documentation)
- Tauri v2 Official Docs (v2.tauri.app) -- architecture, plugins, system tray, IPC
- ElevenLabs API Reference -- streaming endpoints, models, pricing
- Google Cloud TTS REST API -- synthesis endpoint, auth, quotas
- rodio / arboard / enigo crate docs -- API surface and capabilities

### Secondary (community-verified)
- Tauri GitHub Issues (#10025, #12038, #4852, #9326, #9968, #14002) -- platform-specific pitfalls
- Tauri GitHub Discussions (#5624, #7846) -- text capture approaches, secure storage
- Developer blog posts (dev.to, Medium) -- tray app implementation patterns
- Competitor tool READMEs (TinyReadAloud, Natural-Voice-TTS, Vox, TTS-Hotkey, Piper-Tray) -- feature landscape

### Tertiary (needs validation)
- enigo cross-platform behavior -- inferred from docs, not tested
- rodio streaming playback -- API exists but no reference implementation found for this use case
- keyring crate desktop app usage -- documented for CLI tools, less so for Tauri apps

---
*Research completed: 2026-08-15*
*Ready for roadmap: yes*
