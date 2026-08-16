mod capture;
mod config;
mod hotkey;
mod speech;
mod tray;

use std::sync::Mutex;
use tauri::{Manager, WindowEvent};

pub struct AppState {
    pub config: Mutex<config::AppConfig>,
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> config::AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn get_voices() -> Result<Vec<speech::VoiceInfo>, String> {
    speech::get_voices().map_err(|e| e.to_string())
}

#[tauri::command]
fn test_speak() -> Result<(), String> {
    speech::speak("This is a test of ReadToMe text to speech.").map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    new_config: config::AppConfig,
) -> Result<(), String> {
    let mut current = state.config.lock().unwrap();
    let hotkey_changed = current.hotkey != new_config.hotkey;
    let old_hotkey = current.hotkey.clone();

    *current = new_config.clone();
    drop(current);

    config::save_config(&app, &new_config).map_err(|e| e.to_string())?;

    speech::apply_config(&new_config.voice_id, new_config.playback_speed)
        .map_err(|e| e.to_string())?;

    if hotkey_changed {
        if let Err(e) = hotkey::unregister_hotkey(&app, &old_hotkey) {
            log::warn!("Failed to unregister old hotkey: {}", e);
        }
        hotkey::register_hotkey(&app, &new_config.hotkey)
            .map_err(|e| format!("Failed to register new hotkey: {}", e))?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("ReadToMe starting");

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let loaded_config = config::load_config(app.handle());

            if let Err(e) = config::save_config(app.handle(), &loaded_config) {
                log::warn!("Failed to save initial config: {}", e);
            }

            let hotkey_str = loaded_config.hotkey.clone();
            let voice_id = loaded_config.voice_id.clone();
            let speed = loaded_config.playback_speed;

            app.manage(AppState {
                config: Mutex::new(loaded_config),
            });

            #[cfg(target_os = "macos")]
            {
                let trusted = macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
                if trusted {
                    log::info!("macOS Accessibility permission: granted");
                } else {
                    log::warn!("macOS Accessibility permission: not granted — hotkey and text capture require this permission");
                    use tauri_plugin_notification::NotificationExt;
                    let _ = app.handle().notification()
                        .builder()
                        .title("ReadToMe — Permission Required")
                        .body("Enable Accessibility in System Settings > Privacy & Security > Accessibility")
                        .show();
                }
            }

            tray::setup_tray(app.handle())?;

            speech::init().map_err(|e| {
                log::error!("TTS init failed: {}", e);
                anyhow::anyhow!("TTS initialization failed: {}", e)
            })?;

            if let Err(e) = speech::apply_config(&voice_id, speed) {
                log::warn!("Failed to apply TTS config: {}", e);
            }

            hotkey::register_hotkey(app.handle(), &hotkey_str)?;

            log::info!("Setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, save_config, get_voices, test_speak])
        .build(tauri::generate_context!())
        .expect("error building tauri application");

    app.run(|app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            tauri::RunEvent::WindowEvent {
                label,
                event: WindowEvent::CloseRequested { api, .. },
                ..
            } if label == "main" => {
                api.prevent_close();
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            _ => {}
        }
    });
}
