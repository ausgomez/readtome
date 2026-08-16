use tauri::{
    menu::{Menu, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::AppState;

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let hotkey_display = {
        let state = app.state::<AppState>();
        let config = state.config.lock().unwrap();
        config.hotkey.clone()
    };

    let hotkey_item = MenuItemBuilder::new(format!("Hotkey: {}", hotkey_display))
        .enabled(false)
        .build(app)?;
    let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
    let version_item = MenuItemBuilder::new("ReadToMe v0.1.0")
        .enabled(false)
        .build(app)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit ReadToMe").build(app)?;

    let menu = Menu::with_items(app, &[&hotkey_item, &settings_item, &sep1, &version_item, &sep2, &quit])?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)
        .tooltip("ReadToMe - Ready")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => {
                log::info!("Quit requested from tray menu");
                app.exit(0);
            }
            "settings" => {
                show_settings_window(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_settings_window(tray.app_handle());
            }
        })
        .build(app)?;

    log::info!("System tray initialized");
    Ok(())
}

fn show_settings_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
