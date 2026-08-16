# Feature Landscape

**Domain:** Desktop system tray TTS reader (read-selected-text-aloud utility)
**Researched:** 2026-08-15
**Overall confidence:** MEDIUM (cross-verified across multiple competitor READMEs, review sites, and API docs)

## Table Stakes

Features users expect. Missing = product feels incomplete or broken.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Global hotkey to read selected text | The entire product premise. Every competitor (TinyReadAloud, Natural-Voice-TTS, Vox, TTS-Hotkey, Piper-Tray) implements this as their core interaction. | Med | Must work system-wide across browsers, editors, PDF viewers, terminals. Requires clipboard simulation (Ctrl+C send) or OS selection API. |
| System tray presence (no main window) | Users expect this class of tool to be invisible until needed. Balabolka, NaturalReader, TinyReadAloud, Vox all run from tray. A visible window is a dealbreaker for a utility app. | Low | Tauri has system tray support. Right-click context menu expected. |
| Stop playback (same hotkey or dedicated) | Every competitor supports stopping mid-read. Toggle behavior (press to start, press to stop) is the simplest and most expected. TinyReadAloud uses "any key stops." Natural-Voice-TTS uses a separate stop hotkey. | Low | PROJECT.md already specifies toggle model. Good. |
| Voice selection | Every TTS app offers voice choice. NaturalReader: 150+ voices. Vox: per-engine voice lists. Natural-Voice-TTS: 30+ voices. Users expect to pick a voice they find pleasant. | Low | Depends on provider API. ElevenLabs and Google both expose voice lists via API. |
| Playback speed control | Universal across competitors. NaturalReader: up to 3.0x. Natural-Voice-TTS: 0.75x-2.0x. TinyReadAloud: 0.8x-1.5x. Users processing dense text often want 1.5-2x speed. | Low | ElevenLabs does not natively support speed control in the API (must be done client-side via audio processing). Google TTS supports speaking rate via SSML. |
| Settings persistence | Configuration must survive restarts. Every desktop app does this. Losing your voice/hotkey/speed settings between sessions is a bug, not a missing feature. | Low | Store in OS-appropriate config location. Tauri provides app data directories. |
| API key management | Cloud TTS requires API keys. Users expect a settings UI to enter/change keys. Must not leak keys in logs or crash reports. | Low | Secure storage matters. Consider OS keychain integration (Windows Credential Manager, macOS Keychain). |
| Configurable hotkey | Most competitors let users change the trigger key. Default collisions with other apps are common. Ctrl+Alt+R (TinyReadAloud), Ctrl+Win+T (Natural-Voice-TTS), Alt+T (Vox) -- different defaults show the need for customization. | Med | Global hotkey registration can conflict with other apps. Need conflict detection. |
| Visual feedback during reading | Users need to know the app heard them and is working. TinyReadAloud uses a floating status bar with color states. Others use tray icon changes. Without feedback, users mash the hotkey thinking it didn't register. | Low | Tray icon state change (idle vs reading) is minimum. Consider brief notification/tooltip. |
| Multiple TTS provider support | Not every user has the same provider account. Pluggable architecture lets users pick ElevenLabs, Google, or others. Vox supports 3 engines. PROJECT.md already specifies this. | Med | Provider abstraction layer. Each provider has different API shapes, auth, and audio format responses. |
| Auto-start with OS | Utility apps should be available without manual launch. Most tray apps offer this. | Low | OS-specific: Windows startup registry/folder, macOS login items. |

## Differentiators

Features that set ReadToMe apart. Not expected in every competitor, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Streaming audio playback (low latency) | Most small competitors wait for full audio generation before playing. ElevenLabs supports WebSocket streaming with <500ms first-byte latency. Starting playback while audio is still generating makes long text feel instant. This is the single biggest UX differentiator for a cloud TTS app. | High | Requires chunked audio decoding and playback pipeline. ElevenLabs WebSocket API supports this natively. Google TTS can return audio in chunks via streaming RPCs. |
| Intelligent text chunking | Cloud TTS APIs have character limits (~4096 chars for ElevenLabs, ~5000 for Google). When users select a wall of AI-generated text, the app must split it into sentences/paragraphs, synthesize sequentially, and play back seamlessly. Most small competitors silently truncate or fail. | Med | Use sentence boundary detection (period + space). Queue chunks and pre-fetch next while current plays. |
| Clipboard-preserving text capture | Naive approach: simulate Ctrl+C to grab selection, which clobbers the user's clipboard. Better approach: save clipboard contents, grab selection, restore clipboard. Best: use OS selection API (X11 has PRIMARY selection; Windows has no equivalent, must use clipboard). | Med | Critical UX detail. Overwriting clipboard contents silently is a common complaint. Restore-after-capture is expected by power users. |
| Audio output device selection | Let users route TTS audio to specific output (headphones vs speakers). Useful when users want TTS in headphones while other audio goes to speakers. Windows Volume Mixer can do per-app routing, but an in-app selector is more discoverable. | Med | Platform-specific audio device enumeration. Rust audio libraries (cpal, rodio) support device selection. |
| Pause/Resume | PROJECT.md explicitly puts this out of scope for v1 (toggle is play/stop only). However, for longer text passages, pause/resume is highly valued. Natural-Voice-TTS supports Ctrl+Win+Z for pause/resume. Consider for v2. | Med | Requires maintaining playback position state in the audio buffer and chunk queue. |
| Reading progress indicator | Show how far through the text the app has read (e.g., tray tooltip: "Reading... 3/7 chunks"). For long selections, gives users confidence the app is still working and how much remains. | Low | Track chunk index / total chunks. Update tray tooltip or notification. |
| Keyboard shortcut overlay / help | When user right-clicks tray, show current hotkey binding. Reduces "what was my hotkey again?" friction. Tiny feature, outsized UX impact. | Low | Add to context menu. |
| Pronunciation customization | NaturalReader offers a pronunciation editor for names and technical terms. Useful for users reading code-related or domain-specific text. SSML pronunciation hints can be injected. | Med | Requires user-facing dictionary UI and SSML generation. Google TTS supports SSML <phoneme> and <say-as> tags. ElevenLabs has pronunciation dictionaries API. |
| Clipboard monitoring mode | Zabaware and Clipboard TTS auto-read any text copied to clipboard. Useful for "copy and listen" workflows without needing to highlight+hotkey each time. Optional mode, not default. | Med | Clipboard change listener. Must be opt-in to avoid reading passwords, API keys, etc. |
| Multi-language auto-detection | TinyReadAloud auto-detects English vs Spanish and switches voices. For polyglot users or mixed-language content, auto-detection prevents needing to manually switch voices. | Med | Language detection library + per-language voice mapping. Google TTS supports 75+ languages. |

## Anti-Features

Features to explicitly NOT build. These add complexity without value or actively harm the product.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Full document reader / file import | ReadToMe's value is "highlight and listen." Building PDF/DOCX/EPUB import creates a different product (NaturalReader, Speechify) and massively expands scope. Users who want document reading already have those tools. | Stay focused on system-wide text selection. The OS clipboard is the universal interface. |
| Word-by-word text highlighting | Speechify's marquee feature, but requires a visible text window, synchronized timing data, and fundamentally changes the app from invisible-tray-utility to visible-reader-app. Contradicts the "invisible until needed" design principle. | Keep the app windowless during reading. Visual feedback through tray icon states is sufficient. |
| OCR / screenshot-to-text | Speechify offers this. It's a completely separate technology stack (computer vision) that adds enormous complexity. The target user can already select text. | If text isn't selectable (images), users can use dedicated OCR tools. Don't conflate reading and recognition. |
| Built-in text editor / reading pane | Balabolka has a text editor for pasting content. This creates a second workflow that competes with the primary highlight-and-read flow. Maintain one way to do things. | Users paste text in their existing editor and highlight it. |
| Browser extension | PROJECT.md explicitly scopes this as a native system-wide tool. A browser extension is a different distribution model with different constraints. The global hotkey already works in browsers. | System-wide hotkey covers browser use cases without extension installation/maintenance. |
| Offline / local TTS engine (v1) | PROJECT.md puts this out of scope. Local engines (Kokoro, Piper) require model downloads (100MB-2GB), GPU considerations, and produce lower quality than cloud. Competitors like TinyReadAloud and Natural-Voice-TTS focus on local TTS but sacrifice voice quality and add setup complexity. | Cloud-first with pluggable architecture. Local providers can be added as plugins later without changing the core architecture. |
| Reading history / library | Tracking what was read creates privacy concerns and storage overhead. A utility app should be stateless -- read and forget. | No persistence of read text. Treat each reading as ephemeral. |
| Voice cloning | ElevenLabs supports it, but it's a premium feature with ethical concerns, API complexity, and is orthogonal to the core use case. | Use pre-made voices from the provider's library. |
| Dictation / speech-to-text | TinyReadAloud bundles STT. This is a completely separate feature domain. ReadToMe reads TO you, not FROM you. | Keep the product name's promise. One direction only. |
| AI grammar/rephrase features | TinyReadAloud adds Claude-powered grammar correction. Feature creep that dilutes the core value proposition and adds API cost/complexity. | Stay a reader, not a writer's assistant. |
| Cross-device sync | Speechify syncs reading position across devices. ReadToMe is a desktop utility with no reading position to sync. No mobile companion app. | Each platform is standalone. Settings are local. |
| Subscription / account system | For a utility that wraps the user's own API keys, adding accounts and subscriptions adds friction without value. The user already pays their TTS provider. | Ship as a free/open-source tool. Users bring their own API keys. |

## Feature Dependencies

```
Global Hotkey Registration --> Text Capture (hotkey triggers capture)
Text Capture --> TTS Provider Call (captured text sent to API)
TTS Provider Call --> Audio Playback (API returns audio data)
API Key Management --> TTS Provider Call (provider needs auth)

Settings UI --> Voice Selection (voice picker lives in settings)
Settings UI --> Hotkey Configuration (rebind lives in settings)
Settings UI --> API Key Management (key entry lives in settings)
Settings UI --> Provider Selection (choose provider in settings)

Provider Abstraction Layer --> ElevenLabs Provider
Provider Abstraction Layer --> Google TTS Provider
Provider Abstraction Layer --> (future providers)

Text Chunking --> TTS Provider Call (chunks sent individually)
Text Chunking --> Streaming Playback (chunks queued for sequential play)

Clipboard Preservation --> Text Capture (save/restore around capture)

Auto-start --> System Tray (must have tray to auto-start into)
```

## MVP Recommendation

Prioritize (in build order):

1. **System tray with context menu** -- Foundation everything else sits on
2. **Global hotkey registration** -- The trigger mechanism
3. **Text capture from selection** -- The input pipeline
4. **Single TTS provider (ElevenLabs)** -- The output pipeline. Pick ElevenLabs first because streaming support is superior and voice quality is best-in-class. Starter plan ($5/mo) gives API access + 30K chars.
5. **Audio playback** -- Complete the loop: hotkey -> capture -> synthesize -> play
6. **Toggle stop** -- Second press stops playback
7. **Visual feedback** -- Tray icon state changes (idle/reading/error)
8. **Settings window** -- API key entry, voice selection, hotkey config, speed control
9. **Text chunking** -- Handle selections longer than API character limits
10. **Clipboard preservation** -- Save/restore clipboard around text capture

Defer:
- **Google TTS provider**: Add second provider after the architecture is proven with ElevenLabs. Provider abstraction layer should be designed upfront but only one implementation needed for MVP.
- **Streaming playback**: Start with wait-for-full-audio, upgrade to streaming in a fast-follow. Streaming is a high-complexity differentiator best tackled after basic flow works.
- **Pause/Resume**: PROJECT.md explicitly defers this. Toggle (play/stop) is sufficient for v1.
- **Audio output device selection**: Windows per-app routing covers most cases. In-app selection is a polish feature.
- **Multi-language auto-detection**: Start with user-selected voice/language. Auto-detection is a nice-to-have.
- **Clipboard monitoring mode**: Opt-in power-user feature for later.
- **Auto-start with OS**: Low priority for initial release; users can add to startup manually.

## Sources

- [TinyReadAloud (GitHub)](https://github.com/dorofino/TinyReadAloud) -- System tray TTS app with global hotkey, auto language detection, speed control
- [Natural-Voice-TTS (GitHub)](https://github.com/pawelpc/Natural-Voice-TTS) -- System-wide TTS with pause/resume, 30+ voices, speed control
- [Vox (GitHub)](https://github.com/windward47/vox) -- Cross-platform system tray TTS with multiple engine support
- [TTS-Hotkey (GitHub)](https://github.com/KarolStoinski/TTS-Hotkey) -- Minimal hotkey-to-TTS utility
- [Piper Tray (GitHub)](https://github.com/jame25/Piper-Tray) -- System tray TTS using Piper engine
- [NaturalReader vs Balabolka (Speechify)](https://speechify.com/product-reviews/vs/natural-reader-vs-balabolka/) -- Feature comparison
- [Balabolka Review 2026 (CastReader)](https://castreader.ai/blog/balabolka-review) -- Desktop TTS feature overview
- [Speechify Mac App Guide](https://speechify.com/blog/ultimate-guide-to-the-speechify-mac-app-for-text-to-speech/) -- OCR, highlighting, cross-device features
- [ElevenLabs Streaming Docs](https://elevenlabs.io/docs/api-reference/streaming) -- WebSocket and HTTP streaming capabilities
- [ElevenLabs TTS Docs](https://elevenlabs.io/docs/overview/capabilities/text-to-speech) -- API features and capabilities
- [ElevenLabs Pricing (CodaOne)](https://www.codaone.ai/blog/elevenlabs-pricing-guide-2026/) -- Character limits and tier details
- [Google Cloud TTS Pricing (TextToLab)](https://texttolab.com/blog/google-cloud-tts-pricing) -- Voice types, pricing, free tier
- [Best TTS Software 2026 (Guideflow)](https://www.guideflow.com/blog/best-text-to-speech-software) -- Market overview
- [TTS Accessibility Guide (AccessibilityChecker)](https://www.accessibilitychecker.org/blog/text-to-speech-accessibility/) -- Accessibility requirements
- [Clipboard TTS](https://www.clipboardtts.com/) -- Clipboard monitoring auto-read feature
- [tts-joinery (PyPI)](https://pypi.org/project/tts-joinery/1.0.0/) -- Text chunking strategies for TTS APIs
