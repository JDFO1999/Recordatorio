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
use tauri_plugin_notification::NotificationExt;
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let app_handle = app.handle();
            let app_data_dir = get_app_data_dir(&app_handle);
            let db = Database::new(app_data_dir.clone())
                .expect("Failed to initialize database");
            let db = Arc::new(db);
            app.manage(db.clone());

            // Copy default sound if not configured
            if db.get_setting("notification_sound_path")
                .unwrap_or(None)
                .map_or(true, |p| p.is_empty())
            {
                let resource_path = app_handle.path().resource_dir()
                    .ok()
                    .map(|d| d.join("ritmo.wav"));
                if let Some(src) = resource_path {
                    if src.exists() {
                        let dest = app_data_dir.join("ritmo.wav");
                        let _ = std::fs::copy(&src, &dest);
                        let _ = db.set_setting(
                            "notification_sound_path",
                            &dest.to_string_lossy(),
                        );
                    }
                }
            }

            // Pre-request notification permission at startup
            let _ = app_handle.notification().request_permission();

            let default_model_variant = "small".to_string();
            let whisper_engine = Arc::new(WhisperEngine::new(&app_data_dir, &default_model_variant));
            app.manage(whisper_engine);

            tray::setup_tray(&app_handle);
            shortcuts::register_all(&app_handle, &db);

            let handle = app_handle.clone();
            let handle2 = handle.clone();
            let db_clone = db.clone();
            tauri::async_runtime::spawn(async move {
                scheduler::check_overdue_on_startup(&db_clone, &handle).await;
            });
            scheduler::start_scheduler(handle2, db);

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
            commands::get_model_info,
            commands::download_model,
            commands::refresh_shortcuts,
            commands::check_shortcut_conflict,
            commands::get_pending_reminders_count,
            commands::snooze_last_reminder,
            commands::complete_last_reminder,
            commands::save_file,
            commands::delete_file,
            commands::get_db_mode,
            commands::set_db_mode,
            commands::get_sql_server_config,
            commands::test_sql_server_connection,
            commands::save_sql_server_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
