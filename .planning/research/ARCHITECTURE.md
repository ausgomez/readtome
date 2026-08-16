# Architecture Patterns

**Domain:** System tray TTS desktop application (Tauri v2)
**Researched:** 2026-08-15

## Recommended Architecture

ReadToMe follows a **layered Tauri architecture** with clear separation between OS interaction (Rust backend), application logic (Rust services), and minimal UI (web frontend for settings only).

```
+-------------------------------------------------------------------+
|                        System Tray + Menu                         |
|               (TrayIconBuilder, always running)                   |
+-------------------------------------------------------------------+
         |                    |                    |
    Tray Events         Menu Actions        Tooltip Updates
         |                    |                    |
+-------------------------------------------------------------------+
|                     Global Hotkey Listener                        |
|           (tauri-plugin-global-shortcut, Rust-side)               |
+-------------------------------------------------------------------+
         |
    Hotkey Pressed
         |
+-------------------------------------------------------------------+
|                     Text Capture Service                          |
|  1. Save clipboard  2. Simulate Ctrl+C  3. Read clipboard        |
|  4. Restore clipboard                                             |
|           (enigo + arboard, Rust-side)                            |
+-------------------------------------------------------------------+
         |
    Captured Text
         |
+-------------------------------------------------------------------+
|                     TTS Provider Trait                             |
|  trait TtsProvider { async fn synthesize(&self, text) -> Audio }  |
|           (Rust async trait, dynamic dispatch)                    |
+-------------------------------------------------------------------+
    |                                    |
    v                                    v
+-------------------------+  +-------------------------+
| ElevenLabs Provider     |  | Google TTS Provider     |
| POST /v1/tts/.../stream |  | POST /v1/text:synthesize|
| Streaming HTTP chunks   |  | Base64 response decode  |
| reqwest + streaming     |  | reqwest + serde_json    |
+-------------------------+  +-------------------------+
         |                            |
    Audio Bytes (streaming or batch)
         |
+-------------------------------------------------------------------+
|                     Audio Playback Service                        |
|   Sink-based playback with stop control                           |
|           (rodio + cpal, Rust-side)                               |
+-------------------------------------------------------------------+
         |
    System Audio Output
         |
+-------------------------------------------------------------------+
|                     Settings / Config                              |
|   tauri::State<AppConfig> managed state                           |
|   Persisted to disk via serde + JSON file                         |
+-------------------------------------------------------------------+
         |
    #[tauri::command] IPC
         |
+-------------------------------------------------------------------+
|                     Settings UI (Web Frontend)                    |
|   Hotkey config, provider selection, API keys, voice picker       |
|           (HTML/CSS/JS or lightweight framework)                  |
+-------------------------------------------------------------------+
```

### Component Boundaries

| Component | Responsibility | Communicates With | Lives In |
|-----------|---------------|-------------------|----------|
| **Tray Manager** | System tray icon, context menu, tooltip text | Hotkey Listener, Playback Service (for status), Settings | `src-tauri/src/tray.rs` |
| **Hotkey Listener** | Registers and handles global keyboard shortcuts | Text Capture (triggers it), Playback Service (stop toggle) | `src-tauri/src/hotkey.rs` |
| **Text Capture** | Simulates copy, reads clipboard, restores clipboard | Hotkey Listener (triggered by), TTS Pipeline (passes text) | `src-tauri/src/capture.rs` |
| **TTS Provider Trait** | Defines interface for all TTS backends | Text Capture (receives text), Audio Playback (produces audio) | `src-tauri/src/tts/mod.rs` |
| **ElevenLabs Provider** | HTTP streaming TTS via ElevenLabs API | TTS trait (implements it), reqwest (HTTP client) | `src-tauri/src/tts/elevenlabs.rs` |
| **Google TTS Provider** | REST TTS via Google Cloud API | TTS trait (implements it), reqwest (HTTP client) | `src-tauri/src/tts/google.rs` |
| **Audio Playback** | Plays audio bytes, supports stop/cancel | TTS Provider (receives audio), Hotkey Listener (stop signal) | `src-tauri/src/audio.rs` |
| **Config/State** | Persists and manages user settings | All components (read config), Settings UI (write config) | `src-tauri/src/config.rs` |
| **Settings UI** | User-facing settings window | Config/State via Tauri commands (IPC) | `src/` (web frontend) |

### Data Flow

**Primary flow (hotkey-to-speech):**

```
User presses hotkey
  --> Hotkey Listener receives event
    --> Check: is audio currently playing?
      --> YES: Signal Audio Playback to stop. Done.
      --> NO: Continue below.
    --> Text Capture Service:
      1. Save current clipboard contents (arboard)
      2. Simulate Ctrl+C / Cmd+C (enigo)
      3. Brief delay (~50-100ms) for OS to process
      4. Read clipboard (arboard)
      5. Restore original clipboard contents
      6. Return captured text
    --> If text is empty or unchanged: do nothing (no selection)
    --> TTS Provider (selected from config):
      - ElevenLabs: POST streaming request, receive chunked audio bytes
      - Google TTS: POST request, receive base64 audio, decode to bytes
    --> Audio Playback Service:
      - Decode audio format (MP3/WAV/OGG)
      - Create rodio Sink
      - Append audio source to Sink
      - Playback begins (non-blocking, on background thread)
      - Store Sink handle for stop control
    --> Update tray tooltip to "Reading..."
```

**Settings flow (user configures app):**

```
User right-clicks tray --> "Settings" menu item
  --> Open settings WebviewWindow
  --> Frontend loads current config via invoke('get_config')
  --> User changes settings (hotkey, provider, API key, voice)
  --> Frontend calls invoke('save_config', { config })
  --> Rust handler updates tauri::State<AppConfig>
  --> Persists to config JSON file on disk
  --> If hotkey changed: unregister old, register new
  --> If provider changed: update active provider instance
  --> Close window (tray app continues running)
```

## Patterns to Follow

### Pattern 1: Trait-Based Provider Abstraction

**What:** Define a Rust trait for TTS providers so new backends can be added without modifying core logic.

**When:** Always -- this is the core extensibility mechanism.

**Example:**

```rust
use async_trait::async_trait;

#[async_trait]
pub trait TtsProvider: Send + Sync {
    /// Synthesize text to audio bytes.
    /// Returns raw audio data (MP3, WAV, etc.) and the format.
    async fn synthesize(&self, text: &str) -> Result<AudioOutput, TtsError>;

    /// List available voices for this provider.
    async fn list_voices(&self) -> Result<Vec<Voice>, TtsError>;

    /// Provider display name for UI.
    fn name(&self) -> &str;
}

pub struct AudioOutput {
    pub data: Vec<u8>,
    pub format: AudioFormat,
}

pub enum AudioFormat {
    Mp3,
    Wav,
    OggOpus,
}
```

### Pattern 2: Managed State with Tauri

**What:** Use `tauri::State<T>` with `Mutex` or `RwLock` for thread-safe shared state across commands and event handlers.

**When:** For config, active provider reference, and playback state.

**Example:**

```rust
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub playback: Mutex<Option<PlaybackHandle>>,
    pub provider: Mutex<Box<dyn TtsProvider>>,
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(AppConfig::load_or_default()),
            playback: Mutex::new(None),
            provider: Mutex::new(Box::new(ElevenLabsProvider::new())),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            list_voices,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

### Pattern 3: Toggle State Machine for Hotkey

**What:** The hotkey handler acts as a simple state machine: idle -> reading -> idle.

**When:** Every hotkey press.

**Example:**

```rust
fn handle_hotkey(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut playback = state.playback.lock().unwrap();

    if let Some(handle) = playback.take() {
        // Currently playing -> stop
        handle.stop();
        update_tray_tooltip(app, "ReadToMe - Ready");
    } else {
        // Not playing -> capture and read
        drop(playback); // Release lock before async work
        tauri::async_runtime::spawn(async move {
            if let Ok(text) = capture_selected_text() {
                if !text.is_empty() {
                    read_text_aloud(app, text).await;
                }
            }
        });
    }
}
```

### Pattern 4: Clipboard Sandwich

**What:** Save clipboard before simulating copy, restore after reading, to avoid disrupting the user's clipboard.

**When:** Every text capture.

**Example:**

```rust
use arboard::Clipboard;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use std::thread;
use std::time::Duration;

pub fn capture_selected_text() -> Result<String, CaptureError> {
    let mut clipboard = Clipboard::new()?;

    // 1. Save current clipboard
    let original = clipboard.get_text().ok();

    // 2. Simulate Ctrl+C (Windows) or Cmd+C (macOS)
    let mut enigo = Enigo::new(&Settings::default())?;
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press)?;
        enigo.key(Key::Unicode('c'), Direction::Click)?;
        enigo.key(Key::Meta, Direction::Release)?;
    }
    #[cfg(target_os = "windows")]
    {
        enigo.key(Key::Control, Direction::Press)?;
        enigo.key(Key::Unicode('c'), Direction::Click)?;
        enigo.key(Key::Control, Direction::Release)?;
    }

    // 3. Wait for OS to process
    thread::sleep(Duration::from_millis(80));

    // 4. Read clipboard
    let captured = clipboard.get_text()?;

    // 5. Restore original clipboard
    if let Some(orig) = original {
        let _ = clipboard.set_text(&orig);
    }

    Ok(captured)
}
```

### Pattern 5: Rust-Side Hotkey Registration

**What:** Register global shortcuts in Rust setup, not JavaScript. JS keyboard listeners only work when the window has focus, which is never for a tray app.

**When:** App initialization and hotkey reconfiguration.

**Example:**

```rust
use tauri::plugin::global_shortcut::GlobalShortcutExt;

fn setup_hotkey(app: &AppHandle, shortcut_str: &str) -> Result<(), Box<dyn Error>> {
    let shortcut: Shortcut = shortcut_str.parse()?;
    app.global_shortcut().on_shortcut(shortcut, move |app, _scut, _evt| {
        handle_hotkey(app);
    })?;
    Ok(())
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: JavaScript-Side Global Shortcuts

**What:** Using `window.addEventListener('keydown', ...)` for the global hotkey.

**Why bad:** Only fires when the Tauri window has focus. A tray app has no visible window during normal operation, so this approach silently fails 100% of the time.

**Instead:** Use `tauri-plugin-global-shortcut` registered from Rust during app setup.

### Anti-Pattern 2: Synchronous TTS in the Hotkey Handler

**What:** Blocking the hotkey callback thread while waiting for the TTS API response.

**Why bad:** Freezes the entire application. The hotkey listener runs on the main thread. A TTS API call can take 500ms-3s. During this time, the tray becomes unresponsive and the stop toggle cannot fire.

**Instead:** Spawn TTS work on `tauri::async_runtime::spawn()`. Store a handle for cancellation. The hotkey handler checks state and either starts async work or signals stop.

### Anti-Pattern 3: Polling Clipboard in a Loop

**What:** Continuously polling the clipboard for changes to detect when the user copies text.

**Why bad:** High CPU usage, battery drain, interferes with other clipboard operations, and produces false positives from non-ReadToMe copy actions.

**Instead:** Use the hotkey-triggered clipboard sandwich pattern: only read clipboard at the moment the user explicitly requests it.

### Anti-Pattern 4: Storing API Keys in Plain Config Files Without Protection

**What:** Writing API keys to a plain JSON file in the user's config directory.

**Why bad:** Any process can read it. On shared machines, other users may access it.

**Instead:** Use the OS keychain where available (macOS Keychain, Windows Credential Manager) via a crate like `keyring`. For v1, a plain config file is acceptable with a note that keychain integration is a future improvement -- but never log or display the full key in the UI.

### Anti-Pattern 5: One Monolithic lib.rs

**What:** Putting all Rust code in a single `lib.rs` file.

**Why bad:** Quickly becomes unnavigable. Hotkey logic, clipboard manipulation, HTTP clients, audio playback, and config management are distinct concerns.

**Instead:** Use the module structure shown in Component Boundaries above. Each component gets its own file with a clear public API.

## Scalability Considerations

| Concern | Single User (v1) | Future: Multiple Providers | Future: Offline TTS |
|---------|-------------------|---------------------------|---------------------|
| **Provider count** | 2 providers hardcoded in config UI | Dynamic provider registry, load from config | Add local TTS engine as another trait impl |
| **Text length** | Send full text, wait for response | Chunk long text into segments, stream sequentially | Same chunking, but local inference |
| **Audio latency** | Acceptable for short selections (<500 words) | ElevenLabs streaming helps for long text | Near-instant for local models |
| **Config complexity** | Single JSON file | Same file, more provider entries | Same file, model path settings |
| **Memory** | Buffer full audio response in memory | Stream chunks to rodio sink progressively | Same streaming approach |

## Platform-Specific Architecture Notes

### Windows

- **Text capture:** `enigo` simulates Ctrl+C. Works across most applications. Some UWP/modern apps may need fallback approaches.
- **Global hotkeys:** `tauri-plugin-global-shortcut` wraps `RegisterHotKey` Win32 API. Reliable.
- **Audio:** rodio uses WASAPI on Windows by default via cpal.
- **Tray:** Windows system tray is well-supported in Tauri. Left-click and right-click events both work.
- **Autostart:** Can be added later via `tauri-plugin-autostart` or registry entry.

### macOS

- **Text capture:** `enigo` simulates Cmd+C. Requires Accessibility permissions in System Preferences. The app must prompt the user to grant this on first run.
- **Global hotkeys:** Requires Accessibility permissions. Registration may fail silently if not granted.
- **Audio:** rodio uses CoreAudio on macOS via cpal.
- **Tray:** macOS menu bar icon. Menu appears on click (no left/right distinction in menu bar).
- **Permissions:** This is the critical macOS difference -- both hotkey and input simulation require the user to explicitly grant Accessibility access. The app should detect this and guide the user.

## Suggested Build Order

The components have clear dependencies that dictate build order:

```
Phase 1: Foundation
  [Config/State] --> needed by everything
  [Tray Manager]  --> proves app lifecycle works
  Result: App launches to tray, can quit from menu

Phase 2: Core Interaction
  [Global Hotkey] --> depends on: Config (for hotkey binding), Tray (for app handle)
  [Text Capture]  --> depends on: Hotkey (triggers it)
  Result: Hotkey press captures selected text (logged to console)

Phase 3: TTS Integration
  [TTS Provider Trait] --> depends on: nothing (pure abstraction)
  [ElevenLabs Provider] --> depends on: TTS trait, Config (API key)
  Result: Captured text sent to API, audio bytes received

Phase 4: Audio Playback
  [Audio Playback] --> depends on: TTS Provider (audio source)
  [Toggle Logic]   --> depends on: Playback (stop control), Hotkey (trigger)
  Result: Full hotkey -> capture -> TTS -> playback -> stop cycle works

Phase 5: Settings UI
  [Settings Window] --> depends on: Config (read/write), all services (reconfigure)
  [Voice Picker]    --> depends on: TTS Provider (list_voices)
  Result: User can configure everything from the UI

Phase 6: Second Provider + Polish
  [Google TTS Provider] --> depends on: TTS trait (implement it)
  [Error Handling]      --> depends on: all components (surface errors)
  [Tray Status Updates] --> depends on: Playback state
  Result: Complete v1 with two providers
```

**Build order rationale:** Each phase produces a testable, demonstrable increment. Phase 1-2 prove the OS integration works (the riskiest part). Phase 3-4 prove the core value proposition. Phase 5 makes it user-configurable. Phase 6 adds the second provider and polish. This order front-loads risk: if clipboard simulation or hotkey registration fails on a platform, you discover it in Phase 2 before investing in TTS and audio code.

## Sources

- [Tauri v2 Architecture](https://v2.tauri.app/concept/architecture/) - Official architecture documentation
- [Tauri v2 System Tray](https://v2.tauri.app/learn/system-tray/) - Official system tray guide
- [Tauri v2 Calling Rust](https://v2.tauri.app/develop/calling-rust/) - Official IPC/commands documentation
- [Global Shortcuts in Tauri v2](https://dev.to/hiyoyok/global-keyboard-shortcuts-in-tauri-v2-the-right-way-and-the-wrong-way-2h6d) - Best practices article
- [tauri-plugin-global-shortcut](https://docs.rs/crate/tauri-plugin-global-shortcut/latest) - Plugin docs on docs.rs
- [Tauri Discussion #5624](https://github.com/orgs/tauri-apps/discussions/5624) - Selected text capture approaches
- [ElevenLabs Streaming API](https://elevenlabs.io/docs/api-reference/streaming) - Official streaming docs
- [Google Cloud TTS REST API](https://docs.cloud.google.com/text-to-speech/docs/reference/rest) - Official API reference
- [rodio](https://github.com/RustAudio/rodio) - Rust audio playback library
- [enigo](https://github.com/enigo-rs/enigo) - Cross-platform input simulation
- [arboard](https://github.com/1Password/arboard) - Cross-platform clipboard library
