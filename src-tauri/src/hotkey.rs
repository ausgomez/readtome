use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;

use crate::capture::{self, CaptureResult};

pub fn unregister_hotkey(app: &AppHandle, shortcut_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut: Shortcut = shortcut_str.parse().map_err(|e| {
        format!("Invalid shortcut '{}': {}", shortcut_str, e)
    })?;
    app.global_shortcut().unregister(shortcut)?;
    log::info!("Global shortcut unregistered: {}", shortcut_str);
    Ok(())
}

pub fn register_hotkey(app: &AppHandle, shortcut_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut: Shortcut = shortcut_str.parse().map_err(|e| {
        format!("Invalid shortcut '{}': {}", shortcut_str, e)
    })?;

    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(shortcut, move |_app, _scut, event| {
        if event.state == ShortcutState::Pressed {
            log::info!("Hotkey pressed!");
            let handle = app_handle.clone();
            std::thread::spawn(move || {
                match capture::capture_selected_text() {
                    Ok(CaptureResult::Success(text)) => {
                        let preview = if text.len() > 80 {
                            format!("{}...", &text[..80])
                        } else {
                            text.clone()
                        };
                        log::info!("Captured text: {}", preview);

                        let _ = handle.notification()
                            .builder()
                            .title("ReadToMe")
                            .body(&preview)
                            .show();

                        update_tray_tooltip(&handle, &format!("Captured: {}", preview));
                        let reset_handle = handle.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(3));
                            update_tray_tooltip(&reset_handle, "ReadToMe - Ready");
                        });
                    }
                    Ok(CaptureResult::NoSelection) => {
                        log::info!("No text selected");
                        let _ = handle.notification()
                            .builder()
                            .title("ReadToMe")
                            .body("No text captured — try selecting text first")
                            .show();
                        update_tray_tooltip(&handle, "ReadToMe - No text captured");
                        let reset_handle = handle.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(3));
                            update_tray_tooltip(&reset_handle, "ReadToMe - Ready");
                        });
                    }
                    Err(e) => {
                        log::warn!("Capture failed: {}", e);
                        let _ = handle.notification()
                            .builder()
                            .title("ReadToMe")
                            .body(&format!("Capture error: {}", e))
                            .show();
                        update_tray_tooltip(&handle, "ReadToMe - Error");
                        let reset_handle = handle.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(3));
                            update_tray_tooltip(&reset_handle, "ReadToMe - Ready");
                        });
                    }
                }
            });
        }
    })?;

    log::info!("Global shortcut registered: {}", shortcut_str);
    Ok(())
}

fn update_tray_tooltip(app: &AppHandle, tooltip: &str) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}
