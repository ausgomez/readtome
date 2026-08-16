mod capture;
mod config;
mod hotkey;
mod speech;
mod tray;

use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub config: Mutex<config::AppConfig>,
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> config::AppConfig {
    let config = state.config.lock().unwrap();
    let mut masked = config.clone();
    // Mask sensitive fields for frontend display
    if !masked.voice_id.is_empty() {
        masked.voice_id = masked.voice_id.clone();
    }
    masked
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

            // Save defaults on first launch
            if let Err(e) = config::save_config(app.handle(), &loaded_config) {
                log::warn!("Failed to save initial config: {}", e);
            }

            let hotkey_str = loaded_config.hotkey.clone();

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

            hotkey::register_hotkey(app.handle(), &hotkey_str)?;

            log::info!("Setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, save_config])
        .build(tauri::generate_context!())
        .expect("error building tauri application");

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();
        }
    });
}
