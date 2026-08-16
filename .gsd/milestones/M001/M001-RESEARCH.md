# Phase 1: Foundation + Input Pipeline - Research

**Researched:** 2026-08-16
**Domain:** Tauri v2 system tray application with global hotkey and clipboard-based text capture
**Confidence:** MEDIUM

## Summary

Phase 1 delivers the foundational skeleton of ReadToMe: a Tauri v2 app that launches silently to the system tray, registers a global hotkey, captures selected text from any application via the clipboard sandwich pattern, and persists user settings across restarts. This phase addresses five requirements (SYS-01, SYS-03, CAP-01, CAP-02, SET-01) and proves the riskiest piece of the entire project -- system-wide text capture via clipboard simulation.

The implementation uses Tauri v2 (2.11.x) with Rust backend, Svelte 5 frontend (minimal -- only needed for future settings UI), and three key subsystems: (1) system tray via TrayIconBuilder with context menu, (2) global hotkey via tauri-plugin-global-shortcut with Rust-side handler, and (3) text capture via enigo (keyboard simulation) + arboard (clipboard read/write). Settings persistence uses tauri-plugin-store backed by a JSON file.

**Primary recommendation:** Scaffold with `npm create tauri-app@latest` using the Svelte-TypeScript template, then immediately configure the app as tray-only (hidden window, no dock/taskbar icon). Build the clipboard sandwich in Rust with a 100ms delay between simulated copy and clipboard read. Register the global hotkey from Rust setup, not JavaScript.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SYS-01 | App runs as a system tray icon with no main window visible during normal use | TrayIconBuilder API documented with `visible: false` window config and `activationPolicy: "accessory"` for macOS. `RunEvent::ExitRequested` pattern keeps app alive when all windows closed. |
| SYS-03 | User can configure the trigger hotkey to avoid conflicts with other apps | tauri-plugin-global-shortcut v2.3.2 Rust API documented: `register()`, `unregister()`, `with_handler()`. Shortcut struct supports modifiers + key codes. |
| CAP-01 | App captures currently selected/highlighted text from any application system-wide | Clipboard sandwich pattern documented: enigo 0.6.1 simulates Ctrl+C/Cmd+C, arboard 3.6.1 reads clipboard. Timing delay of 80-100ms required. |
| CAP-02 | App preserves clipboard contents by saving before capture and restoring after | arboard `get_text()` saves before simulation, `set_text()` restores after read. Pattern documented in Architecture Patterns section. |
| SET-01 | User settings persist across app restarts | tauri-plugin-store v2.4.x provides `store()` method returning a persistent JSON-backed key-value store. Auto-save with 100ms debounce default. |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| System tray icon and context menu | Native OS (Rust backend) | -- | Tray is an OS-level construct managed by Tauri's Rust runtime, not the webview |
| Global hotkey registration | Native OS (Rust backend) | -- | Must work when app has no focus; JS listeners only fire with window focus |
| Text capture (clipboard sandwich) | Native OS (Rust backend) | -- | Keyboard simulation and clipboard access are OS-level operations via enigo + arboard |
| Settings persistence | Native OS (Rust backend) | Frontend (JS) for future UI | Store plugin works from both Rust and JS; Phase 1 uses Rust-side only |
| Settings UI | Frontend (Svelte) | -- | Deferred to Phase 3; scaffolded but not built in Phase 1 |
| App lifecycle (prevent exit) | Native OS (Rust backend) | -- | RunEvent handler keeps process alive when no windows open |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2.11.5 (crate) | App shell, system tray, IPC | Official Tauri v2 framework. Stable since Oct 2024. `[CITED: v2.tauri.app]` |
| tauri-plugin-global-shortcut | 2.3.2 (crate) | System-wide hotkey registration | Official Tauri plugin. Wraps platform hotkey APIs. `[CITED: v2.tauri.app/plugin/global-shortcut]` |
| tauri-plugin-store | 2.4.3 (crate) / 2.4.4 (npm) | Persist user settings | Official Tauri plugin. JSON-backed key-value store. `[CITED: v2.tauri.app/plugin/store]` |
| enigo | 0.6.1 (crate) | Simulate keyboard input (Ctrl+C/Cmd+C) | Cross-platform keyboard simulation. Supports Key::Control/Meta + Direction::Press/Click/Release. `[CITED: docs.rs/enigo/0.6.1]` |
| arboard | 3.6.1 (crate) | Clipboard read/write in Rust | By 1Password. Cross-platform clipboard with get_text()/set_text()/clear(). `[CITED: docs.rs/arboard/latest]` |
| serde + serde_json | 1.x | Serialization | Required by Tauri IPC and store plugin. `[ASSUMED]` |
| anyhow | 1.x | Error handling | Ergonomic Result types for the backend. `[ASSUMED]` |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tauri-plugin-notification | 2.3.3 (crate) | OS notifications for errors | When text capture fails or for debugging captured text in Phase 1. `[CITED: v2.tauri.app/plugin/notification]` |
| macos-accessibility-client | latest | Check macOS Accessibility permissions | macOS only -- detect if permissions are granted, prompt if not. `[CITED: crates.io/crates/macos-accessibility-client]` |
| tauri-plugin-macos-permissions | 2.3.0 (crate) | Check/request macOS permissions from Tauri | Alternative to macos-accessibility-client with Tauri integration. Provides `checkAccessibilityPermission()` from JS. `[CITED: crates.io/crates/tauri-plugin-macos-permissions]` |

### Frontend (scaffolded but minimal in Phase 1)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Svelte 5 | 5.25+ | Settings UI framework | Scaffolded by create-tauri-app. Minimal use in Phase 1 (placeholder page only). `[ASSUMED]` |
| Vite | 6.x | Build tool / dev server | Bundled with Tauri Svelte template. `[ASSUMED]` |
| TypeScript | 5.x | Type safety | Bundled with Tauri template. `[ASSUMED]` |
| @tauri-apps/api | 2.11.1 (npm) | Tauri JS API bindings | Core Tauri JS API. `[VERIFIED: npm registry]` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| enigo | get-selected-text crate | Thin wrapper, lightly maintained. On Windows does the same Ctrl+C simulation. Better to own the logic. |
| arboard | tauri-plugin-clipboard-manager | Plugin is for JS-side access. Text capture pipeline runs entirely in Rust. |
| tauri-plugin-store | Raw serde file I/O | Plugin handles persistence edge cases (crash safety, path resolution) and integrates with Tauri lifecycle. |
| macos-accessibility-client | tauri-plugin-macos-permissions | Plugin integrates better with Tauri but is a third-party community plugin. macos-accessibility-client is simpler and more direct. |

**Installation:**

Rust (Cargo.toml):
```toml
[dependencies]
tauri = { version = "=2.11.5", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "=2.3.2"
tauri-plugin-store = "=2.4.3"
tauri-plugin-notification = "=2.3.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
enigo = "0.6"
arboard = "3"

[target.'cfg(target_os = "macos")'.dependencies]
macos-accessibility-client = "0"

[build-dependencies]
tauri-build = "=2.11.5"
```

Frontend (npm):
```bash
npm create tauri-app@latest readtome -- --template svelte-ts
npm install @tauri-apps/plugin-global-shortcut@2
npm install @tauri-apps/plugin-store@2
npm install @tauri-apps/plugin-notification@2
```

**Version note:** The existing project research specifies enigo 0.3.x. This is outdated. The latest stable version is 0.6.1 (published Aug 28, 2025). The core API is the same (Key enum, Direction enum, Keyboard trait) but with expanded key variants and improved platform support. Use 0.6.x. `[CITED: docs.rs/crate/enigo/latest]`

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| @tauri-apps/plugin-global-shortcut | npm | 2+ yrs | 75K/wk | github.com/tauri-apps/plugins-workspace | OK | Approved |
| @tauri-apps/plugin-store | npm | 2+ yrs | 203K/wk | github.com/tauri-apps/plugins-workspace | SUS (too-new version) | Approved -- false positive; official Tauri org, 200K+ downloads |
| @tauri-apps/plugin-notification | npm | 2+ yrs | 422K/wk | github.com/tauri-apps/plugins-workspace | OK | Approved |
| @tauri-apps/api | npm | 4+ yrs | 1.7M/wk | github.com/tauri-apps/tauri | OK | Approved |
| create-tauri-app | npm | 4+ yrs | 10K/wk | github.com/tauri-apps/create-tauri-app | OK | Approved |

**Note on @tauri-apps/plugin-store SUS verdict:** The "too-new" flag is a false positive triggered by the latest version (2.4.4) being published 15 days ago. The package is from the official `@tauri-apps` npm organization, has 200K+ weekly downloads, and is hosted in the official Tauri plugins-workspace repository. No human verification checkpoint needed.

**Rust crates (not checked via npm legitimacy tool -- verified via crates.io search results):**
- tauri 2.11.5: Official Tauri crate `[CITED: crates.io/crates/tauri]`
- enigo 0.6.1: Published Aug 2025, 30 versions since 2017 `[CITED: docs.rs/crate/enigo/latest]`
- arboard 3.6.1: By 1Password, published Jun 2026, MIT/Apache-2.0 `[CITED: crates.io/crates/arboard]`

**Packages removed due to SLOP verdict:** none
**Packages flagged as suspicious SUS:** @tauri-apps/plugin-store (false positive, approved)

## Architecture Patterns

### System Architecture Diagram

```
User Presses Global Hotkey
         |
         v
+----------------------------+
| Global Shortcut Handler    |
| (Rust, tauri-plugin-       |
|  global-shortcut)          |
+----------------------------+
         |
         v
+----------------------------+
| Text Capture Service       |  Clipboard Sandwich:
| (Rust: enigo + arboard)    |  1. Save clipboard (arboard)
|                            |  2. Simulate Ctrl+C (enigo)
|                            |  3. Wait 100ms
|                            |  4. Read clipboard (arboard)
|                            |  5. Restore clipboard (arboard)
+----------------------------+
         |
    Captured Text (String)
         |
         v
+----------------------------+
| Phase 1: Log to console /  |  (Future phases: TTS -> Audio)
| Show tray notification     |
+----------------------------+

+----------------------------+
| System Tray                |
| (TrayIconBuilder + Menu)   |
| - Context menu (Quit)      |
| - Tooltip status           |
| - Icon (idle state)        |
+----------------------------+

+----------------------------+
| Settings / Config          |
| (tauri-plugin-store)       |
| - Hotkey binding           |
| - Provider selection       |
| - JSON file on disk        |
+----------------------------+
```

### Recommended Project Structure
```
readtome/
├── src/                         # Frontend (Svelte)
│   ├── App.svelte               # Minimal placeholder
│   ├── main.ts                  # Svelte entry point
│   └── styles.css               # Base styles
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs               # Tauri app builder, plugin init, setup
│   │   ├── tray.rs              # TrayIconBuilder, context menu, events
│   │   ├── hotkey.rs            # Global shortcut registration and handler
│   │   ├── capture.rs           # Clipboard sandwich text capture
│   │   └── config.rs            # AppConfig struct, defaults, store bridge
│   ├── icons/
│   │   ├── icon.png             # App icon (512x512)
│   │   ├── tray-icon.png        # Tray icon (32x32, RGBA, monochrome for macOS template)
│   │   ├── 32x32.png            # Required size
│   │   ├── 128x128.png          # Required size
│   │   └── 128x128@2x.png      # Retina
│   ├── capabilities/
│   │   └── default.json         # Plugin permissions
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
├── package.json
├── vite.config.ts
├── tsconfig.json
└── svelte.config.js
```

### Pattern 1: Tray-Only App Lifecycle

**What:** Configure Tauri to run as a tray-only application with no visible window, no dock/taskbar icon, and prevent exit when windows close.

**When to use:** App initialization.

**Example:**
```rust
// Source: https://v2.tauri.app/learn/system-tray/ + GitHub Discussion #11489
// [CITED: v2.tauri.app/learn/system-tray/]

use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    Manager, RunEvent,
};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Build tray icon with context menu
            let quit_i = MenuItem::with_id(app, "quit", "Quit ReadToMe", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true) // macOS: auto dark/light mode
                .tooltip("ReadToMe - Ready")
                .menu(&menu)
                .show_menu_on_left_click(false) // right-click for menu
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run(|_app, event| {
            // Prevent app from exiting when all windows are closed
            if let RunEvent::ExitRequested { api, .. } = &event {
                api.prevent_exit();
            }
        });
}
```

**tauri.conf.json configuration:**
```json
{
  "productName": "ReadToMe",
  "version": "0.1.0",
  "identifier": "com.readtome.app",
  "build": {
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "ReadToMe",
        "visible": false,
        "skipTaskbar": true,
        "width": 400,
        "height": 300
      }
    ],
    "trayIcon": {
      "id": "main-tray",
      "iconPath": "icons/tray-icon.png",
      "tooltip": "ReadToMe",
      "iconAsTemplate": true
    },
    "macOSPrivateApi": false
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

**macOS dock hiding via tauri.conf.json:**
```json
{
  "app": {
    "macOSPrivateApi": false
  }
}
```
Note: `activationPolicy: "accessory"` can be set in the `app` section to hide the dock icon. `[CITED: dev.to/hiyoyok/building-a-menubar-app-with-tauri-v2-what-nobody-tells-you-2nae]`

### Pattern 2: Global Hotkey Registration (Rust-Side)

**What:** Register global shortcuts in the Rust setup, not JavaScript. JS keyboard listeners only work when the window has focus, which is never for a tray-only app.

**When to use:** App initialization and hotkey reconfiguration.

**Example:**
```rust
// Source: https://v2.tauri.app/plugin/global-shortcut/
// [CITED: v2.tauri.app/plugin/global-shortcut/]

use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

// In setup closure:
let hotkey = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyR);

app.handle().plugin(
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, shortcut, event| {
            if shortcut == &hotkey {
                match event.state() {
                    ShortcutState::Pressed => {
                        // Trigger text capture
                        handle_hotkey_press(app);
                    }
                    ShortcutState::Released => {
                        // No action on release
                    }
                }
            }
        })
        .build(),
)?;

app.global_shortcut().register(hotkey)?;
```

**Capabilities permission (capabilities/default.json):**
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability for ReadToMe",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered",
    "store:default",
    "notification:default"
  ]
}
```

### Pattern 3: Clipboard Sandwich (Text Capture)

**What:** Save clipboard, simulate Ctrl+C/Cmd+C, wait, read clipboard, restore original.

**When to use:** Every text capture triggered by hotkey.

**Example:**
```rust
// Source: .planning/research/ARCHITECTURE.md pattern, verified against
// docs.rs/enigo/0.6.1 and docs.rs/arboard/latest
// [CITED: docs.rs/enigo/0.6.1] [CITED: docs.rs/arboard/latest]

use arboard::Clipboard;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use std::thread;
use std::time::Duration;

pub fn capture_selected_text() -> Result<String, anyhow::Error> {
    let mut clipboard = Clipboard::new()?;

    // 1. Save current clipboard contents
    let original = clipboard.get_text().ok();

    // 2. Clear clipboard so we can detect if copy worked
    let _ = clipboard.clear();

    // 3. Simulate Ctrl+C (Windows/Linux) or Cmd+C (macOS)
    let mut enigo = Enigo::new(&Settings::default())?;

    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press)?;
        enigo.key(Key::Unicode('c'), Direction::Click)?;
        enigo.key(Key::Meta, Direction::Release)?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press)?;
        enigo.key(Key::Unicode('c'), Direction::Click)?;
        enigo.key(Key::Control, Direction::Release)?;
    }

    // 4. Wait for OS to process the copy command
    thread::sleep(Duration::from_millis(100));

    // 5. Read clipboard (the selected text)
    let captured = clipboard.get_text().unwrap_or_default();

    // 6. Restore original clipboard contents
    if let Some(orig) = original {
        let _ = clipboard.set_text(&orig);
    }

    if captured.is_empty() {
        anyhow::bail!("No text was captured - nothing may be selected");
    }

    Ok(captured)
}
```

### Pattern 4: Settings Persistence with tauri-plugin-store

**What:** Use the store plugin to persist user settings as a JSON file.

**When to use:** Loading settings at startup, saving when user changes config.

**Example:**
```rust
// Source: https://v2.tauri.app/plugin/store/
// [CITED: v2.tauri.app/plugin/store/]

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hotkey: String,
    pub provider: String,
    pub voice_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "CmdOrCtrl+Shift+R".to_string(),
            provider: "elevenlabs".to_string(),
            voice_id: "".to_string(),
        }
    }
}

pub fn load_config(app: &tauri::AppHandle) -> Result<AppConfig, anyhow::Error> {
    let store = app.store("settings.json")?;

    let config = AppConfig {
        hotkey: store
            .get("hotkey")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| AppConfig::default().hotkey),
        provider: store
            .get("provider")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| AppConfig::default().provider),
        voice_id: store
            .get("voice_id")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| AppConfig::default().voice_id),
    };

    Ok(config)
}

pub fn save_config(app: &tauri::AppHandle, config: &AppConfig) -> Result<(), anyhow::Error> {
    let store = app.store("settings.json")?;
    store.set("hotkey", json!(config.hotkey));
    store.set("provider", json!(config.provider));
    store.set("voice_id", json!(config.voice_id));
    // Auto-save is enabled by default with 100ms debounce
    Ok(())
}
```

### Anti-Patterns to Avoid

- **JavaScript-side global shortcuts:** `window.addEventListener('keydown')` only fires when the Tauri window has focus. A tray app has no visible window, so this silently fails 100% of the time. Use the Rust-side global-shortcut plugin. `[CITED: v2.tauri.app/plugin/global-shortcut/]`

- **Synchronous clipboard operations in hotkey handler:** The hotkey callback must not block. The clipboard sandwich involves a 100ms sleep -- spawn it on `tauri::async_runtime::spawn()` or `std::thread::spawn()`.

- **Polling clipboard for changes:** High CPU usage, battery drain, false positives. Use hotkey-triggered capture only.

- **Forgetting `RunEvent::ExitRequested` handler:** Without `api.prevent_exit()`, Tauri exits when the last window closes. A tray-only app with `visible: false` has zero visible windows, so it exits immediately on launch. `[CITED: github.com/tauri-apps/tauri/discussions/11489]`

- **One monolithic lib.rs:** Split into tray.rs, hotkey.rs, capture.rs, config.rs from the start.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Global hotkey registration | Raw Win32 RegisterHotKey / macOS CGEvent taps | tauri-plugin-global-shortcut | Cross-platform, handles registration/unregistration, integrates with Tauri event loop |
| Settings persistence | Manual serde file I/O with fsync | tauri-plugin-store | Handles crash safety, correct app data directory, debounced auto-save |
| Clipboard access | Raw Win32 clipboard API / NSPasteboard | arboard crate | Cross-platform, handles format negotiation, by 1Password |
| Keyboard simulation | Raw SendInput / CGEventCreateKeyboardEvent | enigo crate | Cross-platform, handles modifier key state, supports Unicode |
| macOS permission detection | Raw AXIsProcessTrusted FFI call | macos-accessibility-client crate | Wraps the FFI call, includes prompt option |
| System tray | Raw Shell_NotifyIcon / NSStatusItem | Tauri TrayIconBuilder | Integrated with Tauri lifecycle, cross-platform menu support |

**Key insight:** Every OS integration in Phase 1 has a well-maintained crate that handles the platform-specific complexity. Hand-rolling any of these introduces platform bugs that have already been solved.

## Common Pitfalls

### Pitfall 1: Clipboard Clobbering

**What goes wrong:** Text capture via simulated Ctrl+C overwrites the user's clipboard contents. If save/restore fails or has race conditions, users lose their copied data.

**Why it happens:** No cross-platform API exists to read selected text without going through the clipboard.

**How to avoid:**
1. Always save clipboard before simulating copy (arboard `get_text()`)
2. Clear clipboard before simulating copy to detect if copy worked
3. Wait 100ms after simulating copy before reading
4. Restore original clipboard after reading captured text
5. Handle the case where get_text() returns an error (clipboard was empty or contained non-text data)

**Warning signs:** Users report "my clipboard keeps getting cleared."

### Pitfall 2: macOS Accessibility Permissions Silent Failure

**What goes wrong:** Global hotkey and input simulation (enigo) silently do nothing on macOS without Accessibility permissions. No error, no prompt.

**Why it happens:** macOS requires explicit Accessibility permission grants. Unlike camera/microphone, the OS does not always auto-prompt.

**How to avoid:**
1. On first launch (macOS only), check permissions via `macos-accessibility-client::accessibility::application_is_trusted_with_prompt()`
2. If not granted, show a tray notification explaining what to do
3. Include a "Test Hotkey" item in the tray context menu

**Warning signs:** "Hotkey doesn't work" reports from macOS users only.

### Pitfall 3: App Exits Immediately on Launch

**What goes wrong:** The app starts, creates the tray icon, then immediately exits because there are no visible windows and Tauri's default behavior is to exit when the last window closes.

**Why it happens:** With `visible: false` in tauri.conf.json, there are zero visible windows from the start. Tauri interprets this as "all windows closed."

**How to avoid:** Use `.build()` instead of `.run()` on the Builder, then call `.run()` on the built App with a `RunEvent::ExitRequested` handler that calls `api.prevent_exit()`. `[CITED: github.com/tauri-apps/tauri/discussions/11489]`

**Warning signs:** App appears in tray for a split second then disappears.

### Pitfall 4: Hotkey Conflicts with System Shortcuts

**What goes wrong:** Default hotkey collides with an existing system or application shortcut. Either ReadToMe steals it or silently fails to register.

**Why it happens:** Common key combos like Ctrl+R (browser reload) are already taken.

**How to avoid:**
1. Default to a three-key combo: Ctrl+Shift+R (Windows/Linux), Cmd+Shift+R (macOS)
2. Make hotkey configurable from day one (store in settings.json)
3. Detect registration failure and notify user
4. When changing hotkey: unregister old first, then register new

**Warning signs:** "Shortcut doesn't work" or "my browser reload stopped working."

### Pitfall 5: Tray Icon Not Visible on Dark Themes

**What goes wrong:** Tray icon is invisible against a dark taskbar/menu bar because it uses dark colors.

**Why it happens:** Icon was designed for light backgrounds only.

**How to avoid:**
1. Use `icon_as_template(true)` on macOS -- system auto-inverts for dark/light
2. Provide a monochrome icon with alpha transparency (32x32, RGBA)
3. Test on both light and dark system themes

**Warning signs:** Users report "I can't find the app icon."

### Pitfall 6: Text Capture Fails in Terminals

**What goes wrong:** Simulated Ctrl+C sends SIGINT in terminal emulators instead of copying text.

**Why it happens:** Terminals interpret Ctrl+C as interrupt, not copy. Copy is usually Ctrl+Shift+C.

**How to avoid:**
1. Document this as a known limitation
2. Add a fallback: if clipboard content didn't change after simulation, show notification "No text captured -- try Ctrl+Shift+C in terminals"
3. Future: detect active window type and adjust keystroke

**Warning signs:** Users report "it doesn't work in terminal."

### Pitfall 7: skipTaskbar Not Working on Windows (Known Bug)

**What goes wrong:** Setting `skipTaskbar: true` in tauri.conf.json does not actually hide the taskbar entry on Windows.

**Why it happens:** This is a known Tauri v2 bug (Issue #10422). `[CITED: github.com/tauri-apps/tauri/issues/10422]`

**How to avoid:**
1. Set `visible: false` on the window -- if the window is never shown, it may not appear in taskbar
2. If taskbar entry persists, use Rust-side window API: `window.set_skip_taskbar(true)` after setup
3. Monitor the issue for a fix in a future Tauri release

**Warning signs:** ReadToMe appears in Windows taskbar despite being a tray-only app.

## Code Examples

### Scaffolding the Project

```bash
# Source: https://v2.tauri.app/start/create-project/
# [CITED: v2.tauri.app/start/create-project/]

# Interactive mode (recommended for first time):
npm create tauri-app@latest

# Or non-interactive:
npm create tauri-app@latest readtome -- --template svelte-ts

# After scaffolding, add plugins:
cd readtome
npm install @tauri-apps/plugin-global-shortcut@2
npm install @tauri-apps/plugin-store@2
npm install @tauri-apps/plugin-notification@2
```

### Default Hotkey Recommendation

```
Windows/Linux: Ctrl+Shift+R
macOS: Cmd+Shift+R
```

Rationale: Three-key combo avoids most conflicts. "R" for "Read." Ctrl+Shift+R conflicts with hard-refresh in browsers but this is acceptable since hard-refresh is rarely needed and the hotkey is configurable. `[ASSUMED]`

Alternative safe defaults: Ctrl+Shift+Space, Ctrl+Alt+R.

### Tray Icon Specifications

```
Format:   PNG with RGBA (transparency)
Sizes:    32x32 (primary tray size)
Style:    Monochrome with alpha channel (for macOS template mode)
macOS:    Set iconAsTemplate: true in tauri.conf.json for auto dark/light mode
Windows:  Consider providing 16x16 for older Windows versions
```
`[CITED: v2.tauri.app/develop/icons/]`

### Default Config Schema

```json
{
  "hotkey": "CmdOrCtrl+Shift+R",
  "provider": "elevenlabs",
  "api_key": "",
  "voice_id": "",
  "playback_speed": 1.0
}
```
Note: API keys should NOT be stored in tauri-plugin-store in the final product (use OS keychain via `keyring` crate). For Phase 1, storing in the plugin-store is acceptable since there is no TTS integration yet. `[ASSUMED]`

### macOS Accessibility Permission Check

```rust
// Source: https://crates.io/crates/macos-accessibility-client
// [CITED: crates.io/crates/macos-accessibility-client]

#[cfg(target_os = "macos")]
fn check_accessibility_permissions() -> bool {
    macos_accessibility_client::accessibility::application_is_trusted_with_prompt()
}

#[cfg(not(target_os = "macos"))]
fn check_accessibility_permissions() -> bool {
    true // No special permissions needed on Windows/Linux
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| enigo 0.3.x | enigo 0.6.1 | Aug 2025 | Expanded Key enum, better Wayland support, same core API (Key/Direction/Keyboard) |
| Tauri v1 SystemTray API | Tauri v2 TrayIconBuilder | Oct 2024 | Completely different API. v2 uses builder pattern, menu system is separate. |
| tauri-plugin-store v1 | tauri-plugin-store v2 | Oct 2024 | New API: `app.store("file.json")` instead of `StoreBuilder::new()` |
| Manual Cargo.toml features | tauri.conf.json `trayIcon` section | Tauri v2 | Tray icon config is now partly declarative in JSON, not just Rust code |

**Deprecated/outdated:**
- enigo 0.3.x: Superseded by 0.6.1. Key::Layout renamed to Key::Unicode. DSL removed.
- Tauri v1 system tray API: `SystemTray::new()` no longer exists. Use `TrayIconBuilder::new()`.
- Old clipboard crates (clipboard, copypasta): arboard by 1Password is the current standard.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Default hotkey Ctrl+Shift+R / Cmd+Shift+R is a good default | Code Examples | Minor -- hotkey is configurable, user can change if it conflicts |
| A2 | 100ms delay is sufficient for clipboard copy to complete on all platforms | Pattern 3 | Medium -- too short means stale read, too long means perceived lag. May need tuning per platform. |
| A3 | Svelte 5.25+ is the version in the current create-tauri-app template | Standard Stack | Low -- any Svelte 5.x will work |
| A4 | serde 1.x and serde_json 1.x are correct for current Tauri v2 | Standard Stack | Low -- these are extremely stable crates |
| A5 | anyhow 1.x is the right error handling approach | Standard Stack | Low -- standard Rust practice for applications |
| A6 | Storing API keys in tauri-plugin-store is acceptable for Phase 1 | Code Examples | Low -- no TTS integration in Phase 1, keys not used yet |
| A7 | macos-accessibility-client crate version is compatible with current macOS | Supporting Stack | Medium -- Ventura+ has known AXIsProcessTrusted quirks |
| A8 | `activationPolicy: "accessory"` in tauri.conf.json hides macOS dock icon | Pattern 1 | High -- if this config key does not exist in Tauri v2, must use Rust-side API |

## Open Questions

1. **macOS activationPolicy in tauri.conf.json**
   - What we know: Blog posts reference this config key. Tauri v2 docs show it in the config reference.
   - What's unclear: Exact JSON path. Is it `app.macOS.activationPolicy` or `app.macOSPrivateApi`? Need to verify against actual Tauri v2 config schema.
   - Recommendation: Test during scaffolding. If JSON config fails, use the Rust-side API: `app.set_activation_policy(tauri::ActivationPolicy::Accessory)` with `#[cfg(target_os = "macos")]`.

2. **Clipboard sandwich timing**
   - What we know: 50-100ms delay is recommended by multiple sources.
   - What's unclear: Whether 100ms is sufficient on all Windows versions, especially with antivirus software that hooks clipboard operations.
   - Recommendation: Start with 100ms, add a config option for advanced users to increase if needed. Log timing diagnostics during development.

3. **enigo 0.6.x Wayland support**
   - What we know: enigo 0.6.1 has experimental Wayland support via libei.
   - What's unclear: Whether Wayland keyboard simulation works reliably on all Linux distros.
   - Recommendation: Primary target is Windows and macOS. Linux/Wayland is best-effort. X11 works fine.

4. **Window state plugin conflict**
   - What we know: Using `tauri_plugin_window_state` with `RunEvent::ExitRequested` + `prevent_exit()` causes infinite loop on macOS.
   - What's unclear: Whether this affects the latest Tauri v2 versions.
   - Recommendation: Do NOT use tauri-plugin-window-state in Phase 1. Not needed for a tray-only app anyway. `[CITED: github.com/tauri-apps/tauri/discussions/11489]`

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Tauri compilation | **NO** | -- | Must install via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | Frontend build | Yes | v24.11.0 | -- |
| npm | Package management | Yes | 11.6.1 | -- |
| cargo | Rust package management | **NO** | -- | Installed with Rust toolchain |
| Visual Studio Build Tools | Windows Tauri compilation | Not checked (WSL2 environment) | -- | Install if targeting Windows |

**Missing dependencies with no fallback:**
- **Rust toolchain (rustc + cargo):** BLOCKING. Must be installed before any Tauri development can begin. Install via rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

**Missing dependencies with fallback:**
- None

**Note:** The development environment is WSL2 (Linux 6.6.87.2-microsoft-standard-WSL2). Tauri development in WSL2 requires additional system dependencies for the webview (webkit2gtk, etc.). For a Windows-targeting build, consider developing directly on Windows or using cross-compilation.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust) + vitest (frontend, if needed) |
| Config file | none -- see Wave 0 |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Full suite command | `cargo test --manifest-path src-tauri/Cargo.toml && npm run test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SYS-01 | App launches to system tray, no main window visible | manual | Visual inspection after `cargo tauri dev` | N/A |
| SYS-03 | Hotkey configurable | unit | `cargo test config::tests::test_hotkey_persistence` | Wave 0 |
| CAP-01 | Text capture via clipboard sandwich | unit | `cargo test capture::tests::test_capture_text` | Wave 0 |
| CAP-02 | Clipboard preserved after capture | unit | `cargo test capture::tests::test_clipboard_restore` | Wave 0 |
| SET-01 | Settings persist across restarts | unit | `cargo test config::tests::test_settings_roundtrip` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --manifest-path src-tauri/Cargo.toml`
- **Per wave merge:** Full test suite
- **Phase gate:** All tests pass + manual verification of tray icon and hotkey

### Wave 0 Gaps
- [ ] `src-tauri/src/capture.rs` tests -- clipboard sandwich unit tests (mock clipboard)
- [ ] `src-tauri/src/config.rs` tests -- settings load/save roundtrip
- [ ] Cargo test configuration in the Tauri project

Note: System tray visibility and global hotkey functionality require manual testing -- they depend on OS-level integration that cannot be unit tested.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A for Phase 1 |
| V3 Session Management | No | N/A for Phase 1 |
| V4 Access Control | No | N/A for Phase 1 |
| V5 Input Validation | Yes | Validate hotkey string format before registration; validate captured text is non-empty |
| V6 Cryptography | No | N/A for Phase 1 (API key encryption deferred to Phase 3) |

### Known Threat Patterns for Tauri + System Tray

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Clipboard data exposure during sandwich | Information Disclosure | Minimize time between clipboard clear and restore; handle errors to ensure restore always runs |
| Malicious text injection via clipboard | Tampering | Validate/sanitize captured text before passing to TTS in future phases |
| Global shortcut hijacking | Denial of Service | Detect registration failure; allow user to change hotkey |
| Settings file tampering | Tampering | tauri-plugin-store writes to app data directory with user-level permissions |

## Project Constraints (from CLAUDE.md)

1. **Core Value:** One-keypress text-to-speech for any highlighted text, system-wide. If this does not work reliably and instantly, nothing else matters.
2. **Technology Stack:** Tauri v2 + Rust backend + Svelte 5 frontend (locked decisions from CLAUDE.md).
3. **Architecture:** Audio playback via rodio in Rust backend, NOT Web Audio API. Text capture via clipboard simulation (enigo + arboard), NOT accessibility APIs.
4. **Frontend scope:** Settings UI only -- minimal Svelte, no component libraries needed.
5. **GSD Workflow:** All changes must go through GSD commands. No direct repo edits outside a GSD workflow.

## Sources

### Primary (HIGH confidence)
- [Tauri v2 System Tray Guide](https://v2.tauri.app/learn/system-tray/) -- TrayIconBuilder API, menu creation, event handling
- [Tauri v2 Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/) -- Rust-side registration, handler, permissions
- [Tauri v2 Store Plugin](https://v2.tauri.app/plugin/store/) -- Persistent key-value store API
- [Tauri v2 Create Project](https://v2.tauri.app/start/create-project/) -- Scaffolding commands
- [Tauri v2 Configuration Reference](https://v2.tauri.app/reference/config/) -- Window config, tray icon config
- [enigo docs.rs](https://docs.rs/enigo/0.6.1/enigo/) -- Key/Direction/Keyboard API
- [arboard docs.rs](https://docs.rs/arboard/latest/arboard/struct.Clipboard.html) -- Clipboard API

### Secondary (MEDIUM confidence)
- [Tauri Discussion #11489](https://github.com/tauri-apps/tauri/discussions/11489) -- System tray-only app pattern with ExitRequested
- [Building a Menubar App with Tauri v2 (dev.to)](https://dev.to/hiyoyok/building-a-menubar-app-with-tauri-v2-what-nobody-tells-you-2nae) -- macOS dock hiding, activationPolicy
- [macos-accessibility-client crate](https://crates.io/crates/macos-accessibility-client) -- macOS permission detection
- [tauri-plugin-macos-permissions](https://crates.io/crates/tauri-plugin-macos-permissions) -- Tauri-integrated macOS permission checks
- [Tauri Issue #10422](https://github.com/tauri-apps/tauri/issues/10422) -- skipTaskbar bug on Windows
- Project research: .planning/research/ARCHITECTURE.md, PITFALLS.md, STACK.md

### Tertiary (LOW confidence)
- enigo 0.6.x Wayland support -- documented as experimental, not validated
- Clipboard sandwich 100ms timing -- commonly cited but not empirically tested in this session
- skipTaskbar Windows workaround -- suggested based on issue discussion, not verified

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM -- versions verified via npm registry and docs.rs; Rust crate versions cross-checked with web search results
- Architecture: MEDIUM -- patterns sourced from official Tauri docs and verified GitHub discussions
- Pitfalls: MEDIUM -- corroborated across multiple Tauri GitHub issues and developer blogs

**Research date:** 2026-08-16
**Valid until:** 2026-09-16 (30 days -- Tauri v2 ecosystem is stable)
