mod commands;
mod date_parser;
mod db;
mod notifications;
mod scheduler;
mod shortcuts;
mod tray;
mod whisper;

use db::Database;
use std::sync::Arc;
use tauri::Manager;
use whisper::WhisperEngine;

fn get_app_data_dir(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .expect("Failed to get app data directory")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None::<Vec<&str>>,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let app_data_dir = get_app_data_dir(&app.handle());
            let db = Database::new(app_data_dir.clone()).expect("Failed to initialize database");
            let db = Arc::new(db);
            app.manage(db.clone());

            let whisper_engine = Arc::new(WhisperEngine::new());
            app.manage(whisper_engine);

            tray::setup_tray(&app.handle());
            shortcuts::register_all(&app.handle(), &db);

            let handle = app.handle().clone();
            scheduler::check_overdue_on_startup(&db, &handle);
            scheduler::start_scheduler(handle, db);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_reminder,
            commands::create_reminder_from_voice,
            commands::get_pending_reminders,
            commands::get_all_reminders,
            commands::get_reminder_by_id,
            commands::update_reminder,
            commands::mark_completed,
            commands::mark_cancelled,
            commands::snooze_reminder,
            commands::delete_reminder,
            commands::parse_text,
            commands::get_settings,
            commands::get_setting,
            commands::set_setting,
            commands::get_shortcuts,
            commands::update_shortcut,
            commands::test_notification,
            commands::transcribe_audio,
            commands::get_model_status,
            commands::download_model,
            commands::refresh_shortcuts,
            commands::check_shortcut_conflict,
            commands::get_pending_reminders_count,
            commands::snooze_last_reminder,
            commands::complete_last_reminder,
            commands::save_file,
            commands::delete_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
