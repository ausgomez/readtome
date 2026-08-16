# M001: Foundation + Input Pipeline

**Vision:** App runs silently in the system tray and reliably captures selected text from any application on demand. Proves the riskiest piece of the entire project — system-wide text capture via clipboard sandwich — before investing in TTS or audio.

## Success Criteria

- App launches to system tray with no main window or taskbar entry visible
- User can press a configured hotkey and see the currently selected text captured (verified via log output or tray notification)
- After text capture, the user's original clipboard contents are preserved (not overwritten)
- User's hotkey binding and provider settings survive app restart
- User can right-click the tray icon to access a context menu with at minimum a Quit option

## Slices

- [ ] **S01: Tracer: Tray app captures highlighted text via hotkey** `risk:medium` `depends:[]`
  > After this: Launch app, highlight text in any application, press Ctrl+Shift+R, see captured text logged to console. Right-click tray icon, click Quit.

- [ ] **S02: Settings persistence and hotkey reconfiguration** `risk:medium` `depends:[S01]`
  > After this: Change hotkey config value, restart app, verify new hotkey works and old does not. Verify settings.json exists in app data directory.

- [ ] **S03: Capture reliability and platform edge cases** `risk:medium` `depends:[S01,S02]`
  > After this: Press hotkey with no text selected → see notification 'No text captured'. Press hotkey with text selected → see notification with preview. On macOS, first launch prompts for Accessibility permission.

## Boundary Map

Not provided.
<!-- gsd:state-version=4:0 -->
