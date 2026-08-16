use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILENAME: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hotkey: String,
    pub provider: String,
    pub voice_id: String,
    pub playback_speed: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "CmdOrCtrl+Shift+R".to_string(),
            provider: "local".to_string(),
            voice_id: String::new(),
            playback_speed: 1.0,
        }
    }
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    let store = match app.store(STORE_FILENAME) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Failed to open store, using defaults: {}", e);
            return AppConfig::default();
        }
    };

    let hotkey = store
        .get("hotkey")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| AppConfig::default().hotkey);

    let provider = store
        .get("provider")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| AppConfig::default().provider);

    let voice_id = store
        .get("voice_id")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();

    let playback_speed = store
        .get("playback_speed")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0);

    let config = AppConfig {
        hotkey,
        provider,
        voice_id,
        playback_speed,
    };

    log::info!("Config loaded: hotkey={}, provider={}", config.hotkey, config.provider);
    config
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> anyhow::Result<()> {
    let store = app.store(STORE_FILENAME)?;

    store.set("hotkey", serde_json::json!(&config.hotkey));
    store.set("provider", serde_json::json!(&config.provider));
    store.set("voice_id", serde_json::json!(&config.voice_id));
    store.set("playback_speed", serde_json::json!(config.playback_speed));
    store.save()?;

    log::info!("Config saved: hotkey={}, provider={}", config.hotkey, config.provider);
    Ok(())
}
