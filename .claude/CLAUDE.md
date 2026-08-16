<!-- GSD:project-start source:PROJECT.md -->

## Project

**ReadToMe**

A system tray application that reads highlighted text aloud using text-to-speech. The user presses a global hotkey, and whatever text is currently selected anywhere on their system gets read out in a natural-sounding voice. Press the hotkey again to stop. That's the core interaction — invisible until needed, then immediately useful.

**Core Value:** **One-keypress text-to-speech for any highlighted text, system-wide.** If this doesn't work reliably and instantly, nothing else matters.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->

## Technology Stack

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| Tauri | 2.11.5 | App shell, system tray, IPC | Already decided in PROJECT.md. Small binary (~5-10MB), native system tray and global hotkey via official plugins, Rust backend for performance-critical TTS streaming. v2 is mature (stable since Oct 2024, now at 2.11.x). | MEDIUM |
| Rust | 1.77.2+ | Backend logic | Required by Tauri v2 and its plugins. Handles HTTP requests to TTS APIs, audio decoding/playback, text capture, and config persistence. | MEDIUM |

### Frontend (Settings UI)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| Svelte 5 | 5.25+ | Settings window UI | Smallest bundle size of the major frameworks -- critical for a tray app where the UI is a simple settings panel. No virtual DOM overhead. Official Tauri v2 template via `create-tauri-app`. The settings UI is a single window with a few dropdowns and inputs; Svelte's simplicity matches this scope perfectly. React would be overkill; SolidJS has a smaller ecosystem. | MEDIUM |
| Vite | 6.x | Build tool / dev server | Default bundler for Tauri v2's Svelte template. Fast HMR for development. Zero config needed. | MEDIUM |
| TypeScript | 5.x | Type safety for frontend | Catches IPC contract mismatches between Rust and frontend early. Standard in all Tauri templates. | MEDIUM |

### Tauri Official Plugins

| Plugin | Rust Crate Version | NPM Package | Purpose | Why |
|--------|-------------------|-------------|---------|-----|
| global-shortcut | 2.3.2 | @tauri-apps/plugin-global-shortcut | Register system-wide hotkey | Core requirement. Official plugin, well-maintained. Supports shortcut registration/unregistration with handler callbacks. | 
| clipboard-manager | 2.2.2 | @tauri-apps/plugin-clipboard-manager | Read/write system clipboard (JS side) | Provides JS-side clipboard access for settings UI. For Rust-side clipboard operations in the text capture pipeline, use `arboard` directly. |
| store | 2.4.3 | @tauri-apps/plugin-store | Persist user settings (hotkey, provider, API keys, voice) | Key-value store backed by JSON file. Survives app restarts. Better than raw file I/O for settings. |
| notification | 2.3.3 | @tauri-apps/plugin-notification | Notify user of errors (e.g., API key invalid, network error) | Native OS notifications. Nice-to-have for error states. |

### Rust Backend Crates

| Crate | Version | Purpose | Why | Confidence |
|-------|---------|---------|-----|------------|
| reqwest | 0.13.4 | HTTP client for TTS API calls | De facto standard Rust HTTP client. Async with tokio. Supports chunked streaming response via `chunk()` method -- essential for streaming TTS audio. Enable `stream` feature for `bytes_stream()`. | MEDIUM |
| tokio | 1.53+ | Async runtime | Required by reqwest and Tauri's async command system. Use `features = ["full"]` for simplicity in this app. | MEDIUM |
| rodio | 0.21.1 | Audio playback | High-level audio playback built on cpal. Plays decoded audio through system default output. Supports streaming via `Source` trait. Can decode MP3 from in-memory bytes via `Cursor<Vec<u8>>`. 100ms default buffer latency is acceptable for TTS. | MEDIUM |
| serde | 1.0.229 | Serialization framework | Required for Tauri IPC (commands serialize/deserialize via serde). Also used for TTS API request/response JSON. | MEDIUM |
| serde_json | 1.0.151 | JSON serialization | Parse TTS API responses. Construct API request bodies. Config file handling alongside tauri-plugin-store. | MEDIUM |
| anyhow | 1.x | Error handling | Ergonomic error handling for the backend. Simplifies error propagation across TTS provider calls, audio playback, and clipboard operations. | MEDIUM |
| base64 | 0.22.x | Base64 encoding/decoding | Google Cloud TTS API returns audio as base64-encoded string in JSON response. Need to decode to raw bytes before playback. | MEDIUM |
| arboard | 3.x | Clipboard access from Rust | Direct Rust-side clipboard read/write for the clipboard sandwich pattern (save clipboard -> simulate Ctrl+C -> read clipboard -> restore clipboard). Used by tauri-plugin-clipboard-manager internally. Needed because the text capture pipeline runs entirely in Rust, independent of the webview. | MEDIUM |

### Text Capture

| Crate | Version | Purpose | Why | Confidence |
|-------|---------|---------|-----|------------|
| (Manual clipboard approach) | -- | Capture selected text system-wide | Use Tauri's global-shortcut plugin to intercept the hotkey, then simulate Ctrl+C (Windows) / Cmd+C (macOS) via `enigo`, then read clipboard via `arboard` directly in Rust. This is more reliable and maintainable than the `get-selected-text` crate (which is lightly maintained and does essentially the same thing under the hood). | MEDIUM |
| enigo | 0.3.x | Simulate keyboard input | Cross-platform keyboard simulation. Used to programmatically send Ctrl+C/Cmd+C after hotkey press to copy selected text to clipboard. Well-maintained, supports Windows and macOS. | LOW |

### TTS Provider SDKs

| Provider | API Endpoint | Auth | Response Format |
|----------|-------------|------|-----------------|
| ElevenLabs | `POST /v1/text-to-speech/{voice_id}/stream` | `xi-api-key` header | Chunked MP3/PCM stream |
| Google Cloud TTS | `POST /v1/text:synthesize` | Bearer token (API key or OAuth) | JSON with base64-encoded audio |

## Architecture Decisions

### Audio Playback: rodio (Rust backend) over Web Audio API

- The app has no visible window during normal use -- audio must play even when no WebView is active
- rodio plays through the system default output device natively
- No WebView lifecycle dependency -- audio continues even if settings window is closed
- Simpler stop/cancel logic in Rust (drop the audio sink)

### Frontend: Svelte over React

- The entire frontend is a single settings window with ~5 form inputs (hotkey, provider, API key, voice, playback speed)
- Svelte's compiled output is dramatically smaller (matters for a tray app that should feel lightweight)
- No virtual DOM overhead for what is essentially a form
- Official Tauri template support -- zero extra configuration
- React's ecosystem advantages (component libraries, state management) are irrelevant for this UI scope

### Text Capture: Clipboard simulation over Accessibility API

- Works consistently across all applications on both Windows and macOS
- Accessibility API approach (macOS) requires explicit user permission grants and varies by target app
- The `get-selected-text` crate falls back to clipboard simulation anyway on Windows
- Clipboard simulation is the proven approach used by similar tools (e.g., Bob translator, OpenAI translator)
- **Tradeoff:** Briefly overwrites clipboard contents. Mitigate by saving clipboard before capture and restoring after.

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Framework | Tauri v2 | Electron | 150MB+ binary vs ~5-10MB. Overkill for a tray app. Already decided. |
| Framework | Tauri v2 | Native per-platform | Doubles codebase. Tray app has minimal UI surface. |
| Frontend | Svelte 5 | React 19 | Oversized for a settings form. Larger bundle, virtual DOM unnecessary. |
| Frontend | Svelte 5 | SolidJS | Smaller ecosystem, fewer Tauri community examples. Similar performance but less mature tooling. |
| Frontend | Svelte 5 | Vue 3 | Viable alternative, but Svelte produces smaller bundles and has simpler mental model for a small UI. |
| Audio | rodio | cpal (direct) | cpal is low-level -- requires manual PCM handling. rodio abstracts this and adds format decoding. |
| Audio | rodio | Web Audio API | Requires active WebView. Tray app has no window during normal use. |
| HTTP | reqwest | hyper | hyper is lower-level. reqwest wraps hyper with ergonomic API. No need for hyper's granularity. |
| HTTP | reqwest | ureq | ureq is sync-only (blocking). TTS streaming needs async for non-blocking audio pipeline. |
| Text capture | enigo + arboard | get-selected-text crate | Lightly maintained wrapper. On Windows, does the same Ctrl+C simulation. Better to own the logic. |
| Text capture | enigo + arboard | Accessibility API | Platform-specific, permission-heavy, inconsistent across target apps. |
| Clipboard (Rust) | arboard | tauri-plugin-clipboard-manager | Plugin is for JS-side access. Text capture runs entirely in Rust and needs direct arboard access for the save/restore cycle. |
| Config | tauri-plugin-store | Raw serde file I/O | Plugin handles persistence edge cases (crash safety, path resolution) and integrates with Tauri lifecycle. |

## Installation

### Rust (Cargo.toml)

### Frontend (package.json)

# Create project

# Tauri plugin JS bindings

### Dev Dependencies

### System Requirements

- Rust 1.77.2+ (required by Tauri v2 plugins)
- Node.js 18+ (for frontend build)
- Platform SDKs: Visual Studio Build Tools (Windows), Xcode CLI tools (macOS)

## Sources

- Tauri v2 Official Docs: https://v2.tauri.app/
- Tauri v2 Releases: https://v2.tauri.app/release/ (crate versions verified)
- Tauri Plugin Ecosystem: https://v2.tauri.app/plugin/
- Tauri Global Shortcut Plugin: https://v2.tauri.app/plugin/global-shortcut/
- ElevenLabs API Reference: https://elevenlabs.io/docs/api-reference/text-to-speech/stream
- ElevenLabs Models: https://elevenlabs.io/docs/overview/models
- Google Cloud TTS REST API: https://docs.cloud.google.com/text-to-speech/docs/reference/rest/v1/text/synthesize
- rodio crate: https://crates.io/crates/rodio (v0.21.1)
- reqwest crate: https://crates.io/crates/reqwest (v0.13.4)
- serde crate: https://crates.io/crates/serde (v1.0.229)
- arboard crate: https://crates.io/crates/arboard (v3.x, by 1Password)
- get-selected-text crate: https://github.com/yetone/get-selected-text
- CrabNebula on UI libraries for Tauri: https://crabnebula.dev/blog/the-best-ui-libraries-for-cross-platform-apps-with-tauri/
- Tauri System Tray Guide: https://dev.to/hiyoyok/building-a-menubar-app-with-tauri-v2-what-nobody-tells-you-9a2

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
