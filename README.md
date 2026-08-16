# ReadToMe

A system tray application that reads highlighted text aloud using text-to-speech. Press a global hotkey, and whatever text is currently selected anywhere on your system gets spoken. Press again to stop.

Built with [Tauri v2](https://v2.tauri.app/), Rust, and Svelte 5.

## How It Works

1. Select text in any application
2. Press `Ctrl+Shift+R` (configurable)
3. The selected text is read aloud using your system's built-in TTS voices
4. Press the hotkey again to stop

The app lives in your system tray -- no window needed during normal use. A settings panel is available via the tray icon.

## Current Status

- **Text capture:** Working -- copies selected text via clipboard simulation (saves and restores clipboard)
- **Local TTS:** Working -- uses Windows built-in WinRT voices (no API keys needed)
- **Hotkey:** Working -- system-wide `Ctrl+Shift+R`, press to speak / press again to stop
- **Tray icon:** Working -- shows status in tooltip
- **Settings UI:** Basic Svelte settings panel (hotkey, provider, voice, speed)
- **Cloud TTS (ElevenLabs, Google Cloud):** Not yet implemented

## Prerequisites

### Windows (primary target)

- **Rust** -- [rustup.rs](https://rustup.rs) (use the default MSVC toolchain)
- **Visual Studio Build Tools** with "Desktop development with C++" workload
- **Node.js 18+** -- [nodejs.org](https://nodejs.org)
- **WebView2 Runtime** -- pre-installed on Windows 10 21H2+ and Windows 11

### WSL2 (for development from Linux)

All of the above must be installed on the **Windows side**, not inside WSL. The build scripts handle copying the project to the Windows filesystem and invoking the build tools there.

## Getting Started

### From Windows (PowerShell)

```powershell
# Install dependencies
npm install

# Dev mode with hot reload
npx tauri dev

# Release build (produces NSIS + MSI installers)
npx tauri build
```

### From WSL2

```bash
# Dev mode -- syncs to Windows filesystem, launches tauri dev
./scripts/build-windows.sh --dev

# Debug build (faster, unoptimized)
./scripts/build-windows.sh --debug

# Release build (optimized, produces installers)
./scripts/build-windows.sh
```

The WSL build script copies the project to `C:\Users\<you>\readtome-build\` and runs the PowerShell build script there. This is necessary because Tauri's bundler tools can't run from WSL UNC paths.

## Configuration

Settings are persisted in a JSON store and survive restarts. Defaults:

| Setting        | Default            |
|----------------|--------------------|
| Hotkey         | `Ctrl+Shift+R`     |
| TTS Provider   | `local` (system)   |
| Playback Speed | 1.0                |

## Project Structure

```
src/                    # Svelte frontend (settings UI)
src-tauri/
  src/
    lib.rs              # App setup, state management, IPC commands
    main.rs             # Entry point
    capture.rs          # Text capture via clipboard simulation
    config.rs           # Settings persistence (tauri-plugin-store)
    hotkey.rs           # Global shortcut registration and handler
    speech.rs           # Local TTS via system voices (tts crate)
    tray.rs             # System tray icon setup
scripts/
  build-windows.sh      # WSL -> Windows build bridge
  build-windows.ps1     # Windows-native build script
```

## License

MIT
