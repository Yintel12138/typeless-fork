use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Result,
};

pub fn build_tray(app: &AppHandle) -> Result<TrayIcon> {
    let title = MenuItem::with_id(app, "title", "Typeless", false, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&title, &sep1, &settings, &sep2, &quit])?;

    // 1x1 solid RGBA pixel as fallback icon (blue-ish, matches brand colour)
    static FALLBACK_RGBA: &[u8] = &[88, 101, 242, 255];
    let icon = app
        .default_window_icon()
        .cloned()
        .unwrap_or_else(|| Image::new(FALLBACK_RGBA, 1, 1));

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Typeless")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                if let Some(win) = app.get_webview_window("settings") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(tray)
}
