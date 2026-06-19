use tauri::{AppHandle, Emitter, Manager};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

pub fn setup_tray(app: &AppHandle) {
    let show = MenuItemBuilder::with_id("show", "Mostrar/Ocultar")
        .accelerator("Ctrl+Alt+O")
        .build(app)
        .unwrap();

    let new_reminder = MenuItemBuilder::with_id("new_reminder", "Nuevo recordatorio")
        .accelerator("Ctrl+Alt+N")
        .build(app)
        .unwrap();

    let quit = MenuItemBuilder::with_id("quit", "Salir")
        .build(app)
        .unwrap();

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&new_reminder)
        .separator()
        .item(&quit)
        .build()
        .unwrap();

    let tray = app.tray_by_id("main").unwrap();
    tray.set_menu(Some(menu)).unwrap();

    tray.on_menu_event(move |app_handle, event| {
        match event.id().as_ref() {
            "show" => {
                if let Some(window) = app_handle.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
            }
            "new_reminder" => {
                if let Some(window) = app_handle.get_webview_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                let _ = app_handle.emit("global-shortcut:new-reminder", ());
            }
            "quit" => {
                app_handle.exit(0);
            }
            _ => {}
        }
    });
}
