# S02: Settings persistence and hotkey reconfiguration

**Milestone:** M001
**Slice:** S02

**Goal:** Persist user settings (hotkey, provider, voice) via tauri-plugin-store. Load config on startup, apply saved hotkey. Support changing hotkey at runtime.
**Demo:** Change hotkey config value, restart app, verify new hotkey works and old does not. Verify settings.json exists in app data directory.

## Must-Haves

- 1. On first launch, default config (Ctrl+Shift+R, elevenlabs provider) is written to settings.json. 2. Editing settings.json and restarting app applies the new hotkey. 3. Old hotkey no longer triggers capture after change. 4. Config file persists in correct app data directory across restarts.

## Proof Level

- This slice proves: unit tests for config serialization + manual restart test

## Integration Closure

Config module loads on setup, hotkey module reads from config, store plugin persists to disk. Config changes trigger hotkey re-registration.

## Verification

- Log config load/save events with key names (not values for future API keys). Log hotkey registration/unregistration.

<tasks>
- [ ] **T01**: Create AppConfig struct and store integration _(15min)_
  Create config.rs module with AppConfig struct (hotkey: String, provider: String, voice_id: String, playback_speed: f32). Implement Default trait with sensible defaults (hotkey: CmdOrCtrl+Shift+R, provider: elevenlabs, voice_id: empty, playback_speed: 1.0). Implement load_config(app: &AppHandle) -> Result<AppConfig> that reads from tauri-plugin-store settings.json, falling back to defaults for missing keys. Implement save_config(app: &AppHandle, config: &AppConfig) -> Result<()> that writes all fields to the store. Add unit tests for config defaults and serialization roundtrip.
  - Files: `src-tauri/src/config.rs`
  - Verify: cargo test --manifest-path src-tauri/Cargo.toml -- config::tests passes. Config struct serializes to JSON and deserializes back with all fields intact.
- [ ] **T02**: Load config on startup and register saved hotkey _(15min)_
  In lib.rs setup, call config::load_config() to get the persisted (or default) AppConfig. Pass the loaded hotkey string to the hotkey registration logic instead of the hardcoded Ctrl+Shift+R. Parse the hotkey string into a tauri_plugin_global_shortcut::Shortcut. Save default config on first launch (when store file doesn't exist). Store AppConfig in tauri::State via app.manage() for access from other modules.
  - Files: `src-tauri/src/config.rs`
  - Verify: First launch: settings.json created in app data directory with default values. Edit settings.json hotkey field to 'Ctrl+Shift+T', restart app. Ctrl+Shift+T triggers capture, Ctrl+Shift+R does not.
- [ ] **T03**: Add Tauri IPC commands for config get and set _(15min)_
  Add #[tauri::command] functions: get_config(state: State<AppState>) -> AppConfig returns current config (with API key masked). save_config(state: State<AppState>, config: AppConfig) -> Result<()> validates and saves config, triggers hotkey re-registration if hotkey changed (unregister old, register new). Register commands in invoke_handler. These commands prepare for the Phase 3 settings UI but can also be tested via Tauri's invoke mechanism.
  - Files: `src-tauri/src/config.rs`
  - Verify: Tauri commands registered without compilation errors. Integration test: invoke save_config with new hotkey, verify hotkey re-registration logged, verify old hotkey inactive.
</tasks>

## Files Likely Touched

- src-tauri/src/config.rs
<!-- gsd:state-version=3:0 -->
