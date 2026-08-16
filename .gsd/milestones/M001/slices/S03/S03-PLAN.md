# S03: Capture reliability and platform edge cases

**Milestone:** M001
**Slice:** S03

**Goal:** Harden clipboard sandwich (detect failed capture, handle non-text clipboard content), add tray notification for capture result, handle macOS Accessibility permissions, improve tray context menu.
**Demo:** Press hotkey with no text selected → see notification 'No text captured'. Press hotkey with text selected → see notification with preview. On macOS, first launch prompts for Accessibility permission.

## Must-Haves

- 1. Pressing hotkey with no text selected shows a tray notification instead of silent failure. 2. Clipboard containing non-text data (images) is preserved correctly. 3. Tray context menu shows current hotkey binding and app version. 4. On macOS, app checks Accessibility permissions on startup and prompts if not granted. 5. Console log shows timing diagnostics for clipboard sandwich.

## Proof Level

- This slice proves: manual testing of edge cases + unit tests for capture error paths

## Integration Closure

Capture module returns Result with error variants. Tray module displays notifications for success/failure. macOS permission check runs during setup. All new behavior is additive to S01/S02.

## Verification

- Tray notifications for capture success/failure. Timing logs for clipboard sandwich (save/simulate/wait/read/restore durations). macOS permission status logged on startup.

<tasks>
- [ ] **T01**: Add capture failure detection and tray notifications _(15min)_
  Enhance capture.rs to detect when clipboard content did not change after simulated copy (indicates no text was selected). Return a specific error variant for 'no selection detected'. Add tauri-plugin-notification calls: on successful capture, show brief notification with text preview (first 80 chars). On capture failure, show notification 'No text captured — try selecting text first'. On error, show notification with error description. Add notification permissions to capabilities/default.json.
  - Files: `src-tauri/src/capture.rs`
  - Verify: Press hotkey with no text selected -> notification says 'No text captured'. Press hotkey with text selected -> notification shows preview. Press hotkey in terminal (Ctrl+C sends SIGINT) -> notification shows appropriate message.
- [ ] **T02**: Handle non-text clipboard content and add timing diagnostics _(15min)_
  Update clipboard sandwich to handle non-text clipboard content: before simulating copy, check if clipboard contains image or other non-text data. If so, clear clipboard before copy simulation but note that non-text content cannot be restored (log a warning). Add timing instrumentation: measure and log duration of each step (clipboard save, key simulation, wait, clipboard read, clipboard restore). Log total capture time. This data helps tune the 100ms delay per platform.
  - Files: `src-tauri/src/capture.rs`
  - Verify: Copy an image to clipboard, then press hotkey with text selected. Captured text appears in log. Log shows warning about non-text clipboard content. Timing diagnostics appear in console log.
- [ ] **T03**: Enhance tray context menu with hotkey display and version _(10min)_
  Update tray.rs context menu to show: (1) Disabled menu item displaying current hotkey binding (e.g., 'Hotkey: Ctrl+Shift+R'), (2) Separator, (3) 'About ReadToMe v0.1.0' disabled item showing version from Cargo.toml, (4) Separator, (5) 'Quit ReadToMe' action item. Read the current hotkey from AppConfig state. This gives users a way to see their configured hotkey without opening settings.
  - Files: `src-tauri/src/tray.rs`
  - Verify: Right-click tray icon. Menu shows current hotkey binding, version number, and Quit option. Hotkey display matches the configured hotkey in settings.
- [ ] **T04**: Add macOS Accessibility permission check on startup _(15min)_
  Add conditional compilation block for macOS. On app setup, call macos_accessibility_client::accessibility::application_is_trusted_with_prompt() to check and prompt for Accessibility permissions. Log the permission status. If not granted, show a tray notification explaining that the hotkey and text capture require Accessibility permissions with instructions to enable in System Settings > Privacy & Security > Accessibility. Add macos-accessibility-client as a target-specific dependency in Cargo.toml. On non-macOS platforms, this is a no-op.
  - Files: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`
  - Verify: On macOS: first launch prompts for Accessibility permission. Permission status logged to console. On Windows/Linux: no permission check, no errors, compilation succeeds.
</tasks>

## Files Likely Touched

- src-tauri/src/capture.rs
- src-tauri/src/tray.rs
- src-tauri/src/lib.rs
- src-tauri/Cargo.toml
<!-- gsd:state-version=4:0 -->
