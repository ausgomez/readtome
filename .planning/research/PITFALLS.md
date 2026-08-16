# Domain Pitfalls

**Domain:** TTS system tray desktop application (Tauri + cloud TTS)
**Researched:** 2026-08-15
**Overall confidence:** MEDIUM (cross-verified across multiple sources including Tauri GitHub issues, official docs, developer blogs, and ElevenLabs documentation)

## Critical Pitfalls

Mistakes that cause rewrites, broken core functionality, or fundamentally broken UX.

### Pitfall 1: Clipboard Clobbering During Text Capture

**What goes wrong:** The core interaction -- capturing highlighted text from any application -- requires simulating Ctrl+C (Windows) or Cmd+C (macOS) to copy the selection to the clipboard, then reading the clipboard. This overwrites whatever the user previously had on their clipboard. Users lose their clipboard contents every time they use the app.

**Why it happens:** There is no cross-platform OS API to read the current text selection without going through the clipboard. The only reliable approach is to save the current clipboard contents, simulate the copy keystroke, read the new clipboard contents, then restore the original clipboard contents.

**Consequences:** Users lose copied data they were working with. This is especially painful for developers who copy code snippets. If the save/restore cycle has race conditions, data loss becomes intermittent and hard to debug.

**Prevention:**
1. Implement a clipboard save/restore cycle: save clipboard before simulating copy, read selection, restore original clipboard contents
2. Add a small delay (50-100ms) between simulating the keystroke and reading the clipboard -- the copy operation is asynchronous on some platforms
3. Handle multiple clipboard formats (text, rich text, images) in the save/restore cycle, not just plain text
4. Test the timing carefully -- too short a delay and you read stale clipboard data; too long and the user perceives lag

**Detection:** Users report "my clipboard keeps getting cleared" or copied content disappears unpredictably.

**Phase:** Must be addressed in the very first phase that implements text capture. This is foundational.

**Confidence:** MEDIUM -- corroborated across Tauri GitHub discussions (#5624, #7595) and enigo crate documentation.

---

### Pitfall 2: Global Hotkey Silently Fails on macOS Without Accessibility Permissions

**What goes wrong:** The global hotkey works perfectly during development but silently does nothing for end users on macOS. The user presses the hotkey, nothing happens, they think the app is broken. No error message, no prompt, no indication of what went wrong.

**Why it happens:** macOS requires Accessibility permissions for apps that register global keyboard shortcuts when the app does not have focus. Tauri's global-shortcut plugin handles registration but the OS silently blocks the event if permissions are not granted. Unlike camera or microphone permissions, macOS does not always auto-prompt for Accessibility access.

**Consequences:** The entire app is non-functional on macOS for first-time users until they manually navigate to System Settings > Privacy & Security > Accessibility and enable the app. Most users will not figure this out on their own.

**Prevention:**
1. On first launch (macOS only), detect whether Accessibility permissions are granted
2. If not granted, show a clear onboarding dialog explaining why the permission is needed and how to enable it, with a button that opens the correct System Settings pane
3. Provide a visual indicator in the tray menu showing permission status
4. Include a "Test Hotkey" button in settings so users can verify it works after granting permissions

**Detection:** macOS-only support tickets saying "hotkey doesn't work" or "nothing happens when I press the shortcut."

**Phase:** Must be addressed in the phase that implements global hotkey registration. Cannot be deferred -- it blocks all macOS functionality.

**Confidence:** MEDIUM -- confirmed via Tauri GitHub issue #10025, developer blog posts, and the macos-global-hotkey-troubleshooting community skill.

---

### Pitfall 3: Perceivable Latency Gap Between Hotkey Press and Audio Playback

**What goes wrong:** The user presses the hotkey and waits 2-5 seconds before hearing anything. The delay makes the app feel broken or sluggish. Users press the hotkey again thinking it didn't register, which triggers the stop action (toggle behavior), so they end up in a frustrating press-wait-press-nothing loop.

**Why it happens:** The latency chain is longer than developers expect: hotkey detection (10-50ms) + clipboard simulation and read (50-200ms) + network request to TTS API (100-500ms RTT) + TTS synthesis server-side (75-600ms depending on model and region) + audio data transfer (variable) + client-side audio buffer fill (100-500ms) + playback start. Total: easily 500ms-2000ms+ end-to-end, and much worse on first request (cold start) or from geographically distant regions.

**Consequences:** Users perceive the app as broken during the gap. The toggle interaction model (press to start, press to stop) makes this worse because a frustrated re-press cancels the pending speech. The app feels unreliable.

**Prevention:**
1. Show immediate visual feedback when the hotkey is pressed (tray icon change, brief notification, or subtle sound)
2. Use streaming TTS endpoints rather than waiting for full audio generation -- ElevenLabs streaming endpoint returns chunks progressively, reducing time-to-first-byte
3. Use Flash/Turbo models (75ms inference vs 300ms+ for standard models) for lower latency at slight quality cost
4. Implement a "processing" state in the toggle logic: hotkey press 1 = start, press during processing/playing = stop, but don't allow the second press to cancel during the first 500ms (debounce)
5. Pre-warm the TTS connection on app startup (send a lightweight request to avoid cold starts)
6. Choose the nearest API region (ElevenLabs: `api.us.elevenlabs.io` for US users)

**Detection:** Users report "there's a long pause before it starts reading" or "I have to press the hotkey multiple times."

**Phase:** Must be designed for from the start. Streaming architecture is much harder to retrofit than to build initially. The toggle debounce logic should be part of the initial hotkey phase.

**Confidence:** MEDIUM -- corroborated across ElevenLabs official latency docs, Picovoice TTS latency analysis, and Deepgram streaming TTS research.

---

### Pitfall 4: API Keys Exposed in Frontend JavaScript

**What goes wrong:** The TTS API key (ElevenLabs, Google) is stored in a way that is accessible from the frontend JavaScript layer of the Tauri app. Since Tauri bundles a webview, the frontend code is readable by anyone who unpacks the application binary (DMG, MSI, etc.).

**Why it happens:** Developers often store configuration in tauri-plugin-store (which writes an unencrypted or lightly encrypted JSON file) or pass API keys from the frontend to the Rust backend. If the key touches the JS layer at any point, it is extractable.

**Consequences:** Users' API keys can be stolen, leading to unexpected charges on their ElevenLabs/Google Cloud accounts. Even if the user provides their own key (which they should), exposing it through the frontend is a trust violation.

**Prevention:**
1. Store API keys exclusively in the OS credential store using the `keyring` crate (wraps Windows Credential Manager, macOS Keychain, Linux Secret Service with one API)
2. The settings UI should send the API key to the Rust backend via a Tauri command; the backend writes it to the OS keychain and never returns it to the frontend
3. After initial entry, the settings UI should show a masked placeholder, never the actual key
4. All TTS API calls must originate from the Rust backend, never from frontend JavaScript
5. Do NOT use tauri-plugin-store for secrets -- it is designed for preferences, not credentials

**Detection:** Security audit finds API keys in the app bundle, in localStorage, or in plaintext config files.

**Phase:** Must be addressed in the phase that implements settings/configuration, before TTS provider integration. The storage mechanism must be in place before the first API key is accepted.

**Confidence:** MEDIUM -- corroborated across Tauri community discussions (#7846), developer blog posts, and keyring crate documentation.

---

### Pitfall 5: Audio Playback Works in Dev but Breaks in Production Builds

**What goes wrong:** Audio plays perfectly during `tauri dev` but produces silence or errors after building and installing the production binary. This is one of the most commonly reported Tauri audio issues.

**Why it happens:** Multiple causes compound:
- HTML5 Audio element with remote URLs may be blocked by Content Security Policy (CSP) in production builds
- Tauri's webview handles audio differently in dev mode vs production mode
- Windows WebView2 may block autoplay without prior user interaction
- On macOS, ATS (App Transport Security) blocks HTTP URLs; only HTTPS works in production
- Audio format support varies by platform webview (e.g., .ogg not supported on Safari/WebKit)

**Consequences:** The app ships broken. Everything looks fine during development, then users report "no sound" after installation.

**Prevention:**
1. Handle audio playback in the Rust backend using `rodio` + `cpal`, not in the frontend webview -- this eliminates all webview-related audio issues
2. If using webview audio, test specifically with production builds (`tauri build`) during development, not just `tauri dev`
3. Configure CSP to allow the TTS API domains explicitly
4. Use only HTTPS endpoints for all API calls
5. Use MP3 format (universally supported) rather than OGG or WAV

**Detection:** "Works on my machine during development" but user reports of silence or audio errors in installed builds.

**Phase:** Must be decided in the architecture phase. Choosing Rust-side audio (rodio) vs webview audio (HTML5 Audio) is an architectural decision that is expensive to change later.

**Confidence:** MEDIUM -- confirmed via multiple Tauri GitHub issues (#4506, #9326, #9968, #14002) documenting this exact pattern.

## Moderate Pitfalls

### Pitfall 6: ElevenLabs Character Limits and Unexpected Costs

**What goes wrong:** Users paste or highlight very long documents and hit the per-request character limit (5,000 chars on paid plans, 2,500 on free tier), causing the API to reject the request. Or they use the app heavily and exceed their monthly quota, incurring overage charges at $0.30/1,000 characters (3x the normal rate).

**Prevention:**
1. Implement text length validation before sending to the API -- show a clear error if the selection exceeds the limit
2. For long texts, split into chunks at sentence boundaries and send sequentially, playing each chunk as it arrives
3. Display a character/usage counter in the settings UI so users can monitor their consumption
4. Optionally warn when approaching the monthly quota limit (track usage locally)
5. Document character limits clearly in the settings UI near the API key entry

**Detection:** Users report "it only reads part of my text" or "my ElevenLabs bill was way higher than expected."

**Phase:** TTS provider integration phase. Must be part of the provider abstraction layer.

**Confidence:** MEDIUM -- confirmed via ElevenLabs pricing documentation and API reference.

---

### Pitfall 7: Registering the Hotkey Conflicts with System or Application Shortcuts

**What goes wrong:** The default hotkey conflicts with an existing system shortcut or a shortcut in the user's most-used application. The app either steals the shortcut (breaking the other app) or silently fails to register (the other app keeps it).

**Prevention:**
1. Choose a default hotkey that is unlikely to conflict -- use a three-key combination (e.g., Ctrl+Shift+R on Windows, Cmd+Shift+R on macOS) rather than two-key
2. Make the hotkey fully configurable from the settings UI from day one
3. Detect registration failure and notify the user with a suggestion to change the shortcut
4. When re-registering a changed shortcut, always unregister the old one first -- double registration causes silent failures on macOS
5. Use platform-appropriate modifier keys: Ctrl on Windows/Linux, Cmd on macOS

**Detection:** Users report "the shortcut doesn't work" or "my other app's shortcut stopped working."

**Phase:** Global hotkey registration phase. Configurability is not a nice-to-have -- it is essential.

**Confidence:** MEDIUM -- confirmed via Tauri global-shortcut plugin documentation and GitHub issues.

---

### Pitfall 8: System Tray Icon Not Visible or Behaves Differently Across Platforms

**What goes wrong:** The tray icon looks wrong, is invisible, or behaves unexpectedly on different platforms. The app window shows in the taskbar/dock even though it should be tray-only.

**Prevention:**
1. Provide platform-specific icon sizes: 16x16 for Windows, 22x22@2x (44x44 actual) for macOS, multiple sizes for Linux
2. Use PNG format with transparency for all platforms
3. On macOS, use `tauri::ActivationPolicy::Accessory` to hide the dock icon; use `.build()` instead of `.run()` to access the app handle
4. On Windows, use `.skip_taskbar(true)` on the window builder to hide the taskbar entry
5. Test tray icon visibility on both light and dark system themes -- provide both light and dark icon variants or use a template icon on macOS
6. On macOS, use template images (monochrome with alpha) so the system automatically adapts to light/dark menu bar

**Detection:** Users report "I can't find the app" or "the icon is invisible on my dark taskbar."

**Phase:** System tray setup phase. Icon handling must be correct from the start.

**Confidence:** MEDIUM -- confirmed via Tauri GitHub issue #4852 and tray implementation guides.

---

### Pitfall 9: Text Capture Fails in Certain Applications

**What goes wrong:** The simulated Ctrl+C/Cmd+C does not work in all applications. Terminal emulators use Ctrl+C for interrupt (SIGINT), not copy. PDF viewers may not support standard copy. Some Electron apps intercept copy differently. Web apps in browsers may have custom copy handlers.

**Prevention:**
1. Document known limitations -- terminal applications are the most obvious case
2. Add a fallback: if clipboard content does not change after the simulated keystroke, show a notification saying "No text was captured. Try selecting text and copying it manually first."
3. Consider an alternative capture mode: read from clipboard directly (user copies manually, then presses the hotkey to read what's on the clipboard) as a fallback
4. Test against common applications: browsers (Chrome, Firefox, Edge), VS Code, terminal emulators, PDF viewers, Microsoft Office, LibreOffice

**Detection:** Users report "it doesn't work in [specific app]" with Terminal being the most common.

**Phase:** Text capture phase. The fallback mode should be designed alongside the primary capture mechanism.

**Confidence:** MEDIUM -- based on Tauri GitHub discussion #5624 and general clipboard/input simulation knowledge.

---

### Pitfall 10: WebSocket Connection Management for Streaming TTS

**What goes wrong:** The ElevenLabs WebSocket connection drops unexpectedly, audio cuts off mid-sentence, or the app leaks connections. The WebSocket auto-closes after 20 seconds of inactivity, so if the user doesn't trigger another read within that window, the next request has to re-establish the connection (adding latency).

**Prevention:**
1. Use HTTP streaming endpoint (Server-Sent Events) for the initial implementation -- it is simpler and more reliable than WebSockets for a "read selected text" use case where full text is available upfront
2. If using WebSockets, send keepalive pings (a single space character) to prevent the 20-second timeout
3. Implement automatic reconnection with exponential backoff
4. Handle connection drops mid-audio gracefully -- buffer enough audio to cover brief reconnection periods
5. Enable `auto_mode` on the WebSocket to avoid chunk scheduling stalls

**Detection:** Audio cuts off mid-sentence, or there's a long delay on the second use after a period of inactivity.

**Phase:** TTS provider integration phase. HTTP streaming should be the initial approach; WebSockets can be added later as an optimization.

**Confidence:** MEDIUM -- confirmed via ElevenLabs streaming and WebSocket documentation.

## Minor Pitfalls

### Pitfall 11: Google Cloud TTS Authentication Complexity

**What goes wrong:** Google Cloud TTS requires a service account JSON key file and the GOOGLE_APPLICATION_CREDENTIALS environment variable, which is complex for end users to set up compared to a simple API key.

**Prevention:**
1. For the Google TTS provider, accept the service account JSON content directly in the settings UI and store it securely (not just the file path)
2. Alternatively, use Google Cloud API keys (simpler but less secure) instead of service account authentication for client-side desktop apps
3. Provide clear setup instructions with screenshots in the settings UI
4. Consider making ElevenLabs the primary/recommended provider since its setup is simpler (just an API key string)

**Detection:** Users struggle to configure Google TTS or report "authentication failed" errors.

**Phase:** Google TTS provider implementation phase.

**Confidence:** MEDIUM -- based on Google Cloud TTS documentation and common developer experience.

---

### Pitfall 12: Tauri v2 API Instability and Breaking Changes

**What goes wrong:** Tauri v2 APIs change between minor versions, breaking existing code. The system tray API changed significantly between v1 and v2, and the plugin ecosystem (global-shortcut, clipboard, store) has its own versioning.

**Prevention:**
1. Pin all Tauri dependencies to exact versions in Cargo.toml and package.json
2. Use `=` version pinning, not `^` or `~`, for Tauri core and all plugins
3. Test builds after any dependency update before committing
4. Follow the Tauri v2 migration guide if starting from v1 examples or tutorials
5. Monitor the Tauri GitHub releases for breaking changes

**Detection:** Build failures or runtime errors after running `cargo update` or `npm update`.

**Phase:** Project setup phase. Version pinning should be established from the first commit.

**Confidence:** MEDIUM -- based on Tauri community discussions and v1-to-v2 migration issues.

---

### Pitfall 13: No Feedback When TTS Provider is Misconfigured or Unreachable

**What goes wrong:** The user has an invalid API key, no internet connection, or the TTS service is down. They press the hotkey, nothing happens, and they have no idea why.

**Prevention:**
1. Validate the API key on entry (make a test API call with a short text like "test")
2. Show a tray icon state change for errors (red dot, warning icon)
3. Use system notifications for errors: "Could not connect to ElevenLabs" or "Invalid API key"
4. Implement a health check on app startup that verifies the TTS provider is reachable
5. Show the last error in the tray menu or settings window for debugging

**Detection:** Users report "the app just does nothing" without any error indication.

**Phase:** Should be part of both the settings phase (key validation) and the TTS integration phase (runtime error handling).

**Confidence:** MEDIUM -- general UX best practice, corroborated by the project's core value statement that reliability is paramount.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Project setup / scaffolding | Tauri v2 version drift (#12) | Pin all deps to exact versions from day one |
| System tray implementation | Icon not visible on dark themes (#8) | Provide template icons, test both themes |
| System tray implementation | App shows in dock/taskbar (#8) | Use ActivationPolicy::Accessory (macOS) + skip_taskbar (Windows) |
| Global hotkey registration | macOS Accessibility permissions (#2) | Detect and guide user through permission grant on first launch |
| Global hotkey registration | Shortcut conflicts (#7) | Make configurable from the start, use three-key combo default |
| Text capture | Clipboard clobbering (#1) | Save/restore clipboard, add timing delays |
| Text capture | Fails in terminals and some apps (#9) | Implement clipboard-direct fallback mode |
| TTS provider integration | Latency gap kills UX (#3) | Use streaming endpoints, Flash models, visual feedback |
| TTS provider integration | Character limits cause silent failures (#6) | Validate length, split at sentence boundaries |
| TTS provider integration | Connection management (#10) | Start with HTTP streaming, not WebSockets |
| API key / settings | Keys exposed in frontend (#4) | Store in OS keychain via keyring crate, never touch JS |
| Audio playback | Works in dev, breaks in production (#5) | Use Rust-side rodio, not webview HTML5 Audio |
| Error handling | Silent failures everywhere (#13) | Tray icon states, system notifications, key validation |

## Sources

- [Tauri GitHub Issue #10025: Global shortcut fires twice on macOS](https://github.com/tauri-apps/tauri/issues/10025) [MEDIUM]
- [Tauri GitHub Issue #12038: Ctrl+C shortcut breaks copy](https://github.com/tauri-apps/tauri/issues/12038) [MEDIUM]
- [Tauri GitHub Discussion #5624: Getting user selected text](https://github.com/tauri-apps/tauri/discussions/5624) [MEDIUM]
- [Tauri GitHub Discussion #7846: Secure storage for secrets](https://github.com/tauri-apps/tauri/discussions/7846) [MEDIUM]
- [Tauri GitHub Issue #4852: Hide taskbar icon on macOS](https://github.com/tauri-apps/tauri/issues/4852) [MEDIUM]
- [Tauri GitHub Issue #9326: Can't play MP3](https://github.com/tauri-apps/tauri/issues/9326) [MEDIUM]
- [Tauri GitHub Issue #9968: Audio autoplay not working on Windows](https://github.com/tauri-apps/tauri/issues/9968) [MEDIUM]
- [Tauri GitHub Issue #14002: Play remote audio](https://github.com/tauri-apps/tauri/issues/14002) [MEDIUM]
- [Tauri v2 Global Shortcut Plugin docs](https://v2.tauri.app/plugin/global-shortcut/) [MEDIUM]
- [ElevenLabs Latency Optimization Guide](https://elevenlabs.io/docs/eleven-api/guides/how-to/best-practices/latency-optimization) [MEDIUM]
- [ElevenLabs Audio Streaming Concepts](https://elevenlabs.io/docs/eleven-api/concepts/audio-streaming) [MEDIUM]
- [ElevenLabs API Pricing](https://elevenlabs.io/pricing/api) [MEDIUM]
- [Google Cloud TTS Quotas](https://docs.cloud.google.com/text-to-speech/quotas) [MEDIUM]
- [Picovoice TTS Latency Analysis](https://picovoice.ai/blog/text-to-speech-latency/) [MEDIUM]
- [Global Keyboard Shortcuts in Tauri v2 (dev.to)](https://dev.to/hiyoyok/global-keyboard-shortcuts-in-tauri-v2-the-right-way-and-the-wrong-way-43a2) [MEDIUM]
- [Storing API Keys Securely in Tauri (dev.to)](https://dev.to/hiyoyok/storing-a-gemini-api-key-securely-in-a-tauri-app-dont-hardcode-it-4cdk) [MEDIUM]
- [System Tray Tauri v2 Implementation (Medium)](https://medium.com/@sjobeiri/understanding-the-system-tray-from-concept-to-tauri-v2-implementation-252f278bb57c) [MEDIUM]
- [rodio - Rust audio playback library](https://github.com/RustAudio/rodio) [MEDIUM]
- [keyring crate - OS credential store wrapper](https://crates.io/crates/keyring) [MEDIUM]
