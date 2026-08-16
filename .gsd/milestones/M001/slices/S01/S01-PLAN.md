# S01: Tracer: Tray app captures highlighted text via hotkey

**Milestone:** M001
**Slice:** S01

**Goal:** Scaffold Tauri v2 project, launch as tray-only app with context menu, register global hotkey, capture selected text via clipboard sandwich, log captured text. Proves the entire input pipeline end-to-end.
**Demo:** Launch app, highlight text in any application, press Ctrl+Shift+R, see captured text logged to console. Right-click tray icon, click Quit.

## Must-Haves

- 1. cargo tauri dev launches app to system tray with no visible window. 2. Right-click tray shows context menu with Quit. 3. Press Ctrl+Shift+R with text selected → captured text appears in console log. 4. Clipboard content before capture matches clipboard content after capture. 5. Click Quit from tray menu exits the app cleanly.

## Proof Level

- This slice proves: manual + unit tests for capture logic

## Integration Closure

Tray icon appears in system tray. Hotkey handler calls capture module. Capture module uses enigo + arboard. All Rust-side, no frontend dependency.

## Verification

- Console logging of captured text, hotkey events, and clipboard save/restore steps. Tray tooltip shows app state.

<tasks>
- [ ] **T01**: Install Rust toolchain and scaffold Tauri v2 project _(30min)_
  Install Rust via rustup if not present. Install system dependencies for Tauri on the current platform (WSL2: webkit2gtk, libayatana-appindicator, etc.). Scaffold the project with `npm create tauri-app@latest` using the svelte-ts template. Add Rust dependencies to Cargo.toml: tauri with tray-icon feature, tauri-plugin-global-shortcut, tauri-plugin-store, tauri-plugin-notification, serde, serde_json, anyhow, enigo, arboard. Add npm dependencies: @tauri-apps/plugin-global-shortcut, @tauri-apps/plugin-store, @tauri-apps/plugin-notification. Pin Tauri crate versions with = prefix. Verify the project compiles with `cargo build`.
  - Files: `src-tauri/Cargo.toml`, `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`
  - Verify: cargo build --manifest-path src-tauri/Cargo.toml succeeds with exit code 0. npm install completes without errors.
- [ ] **T02**: Configure tray-only app lifecycle with system tray icon and context menu _(20min)_
  Configure tauri.conf.json: set window visible=false, skipTaskbar=true. Create tray.rs module with TrayIconBuilder setup: context menu with Quit item, icon_as_template(true) for macOS, tooltip 'ReadToMe - Ready', menu event handler for quit. In lib.rs, use .build() + .run() with RunEvent::ExitRequested handler calling api.prevent_exit(). Register all plugins (global-shortcut, store, notification). Create a simple monochrome tray icon PNG (32x32). Configure capabilities/default.json with required plugin permissions.
  - Files: `src-tauri/src/lib.rs`, `src-tauri/src/tray.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/icons/tray-icon.png`
  - Verify: cargo tauri dev launches app. System tray icon appears. Right-click shows context menu with 'Quit ReadToMe'. Clicking Quit exits the process. No main window or taskbar entry is visible.
- [ ] **T03**: Register global hotkey with Rust-side handler _(15min)_
  Create hotkey.rs module. In the setup closure, create a Shortcut for Ctrl+Shift+R (default). Register it via app.global_shortcut().register(). Add a handler via tauri_plugin_global_shortcut::Builder::new().with_handler() that logs 'Hotkey pressed!' to console on ShortcutState::Pressed. Wire the hotkey module into lib.rs setup. Ensure the handler runs on the correct thread (not blocking the main thread).
  - Files: `src-tauri/src/hotkey.rs`, `src-tauri/src/lib.rs`
  - Verify: cargo tauri dev running. Press Ctrl+Shift+R. Console shows 'Hotkey pressed!' log message. Pressing other key combinations does not trigger the handler.
- [ ] **T04**: Implement clipboard sandwich text capture _(20min)_
  Create capture.rs module with pub fn capture_selected_text() -> Result<String, anyhow::Error>. Implementation: (1) Create arboard::Clipboard, (2) save current clipboard via get_text().ok(), (3) clear clipboard, (4) create enigo instance, (5) simulate Ctrl+C (Windows/Linux) or Cmd+C (macOS) using platform-conditional compilation, (6) sleep 100ms for OS processing, (7) read clipboard via get_text(), (8) restore original clipboard content, (9) return captured text or error if empty. Add logging at each step for diagnostics. Handle errors with anyhow context.
  - Files: `src-tauri/src/capture.rs`, `src-tauri/src/lib.rs`
  - Verify: Unit test: mock clipboard save/restore cycle. Manual test: select text in browser, call capture function, verify returned string matches selection, verify clipboard restored.
- [ ] **T05**: Wire hotkey handler to text capture and log output _(15min)_
  Connect the hotkey handler in hotkey.rs to call capture::capture_selected_text(). Since the clipboard sandwich blocks (100ms sleep), spawn the capture on a background thread via std::thread::spawn or tauri::async_runtime::spawn_blocking. On success, log the captured text to console with println! or tracing. On failure, log the error. Update tray tooltip to show 'Captured: {first 50 chars}...' after successful capture, then reset to 'ReadToMe - Ready' after 3 seconds. This completes the end-to-end tracer: hotkey -> capture -> log.
  - Files: `src-tauri/src/hotkey.rs`, `src-tauri/src/capture.rs`, `src-tauri/src/tray.rs`
  - Verify: cargo tauri dev running. Select text in any application. Press Ctrl+Shift+R. Console shows captured text. Tray tooltip briefly shows captured text preview. Original clipboard content is unchanged.
</tasks>

## Files Likely Touched

- src-tauri/Cargo.toml
- package.json
- src-tauri/tauri.conf.json
- src-tauri/src/lib.rs
- src-tauri/src/tray.rs
- src-tauri/capabilities/default.json
- src-tauri/icons/tray-icon.png
- src-tauri/src/hotkey.rs
- src-tauri/src/capture.rs
<!-- gsd:state-version=2:0 -->
