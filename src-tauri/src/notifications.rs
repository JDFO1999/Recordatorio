use std::process::Command as StdCommand;
use tauri::AppHandle;
use tauri::Runtime;
use tauri_plugin_notification::{NotificationExt, PermissionState};

fn is_file_path(path: &str) -> bool {
    path.contains('\\') || path.contains('/') || path.ends_with(".wav")
}

fn play_sound_file(path: &str) {
    let escaped = path.replace('\'', "''");
    StdCommand::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("(New-Object Media.SoundPlayer '{}').PlaySync()", escaped),
        ])
        .spawn()
        .ok();
}

fn add_sound<R: Runtime>(
    builder: tauri_plugin_notification::NotificationBuilder<R>,
    sound_path: Option<&str>,
) -> tauri_plugin_notification::NotificationBuilder<R> {
    if let Some(path) = sound_path {
        if !path.is_empty() {
            if is_file_path(path) {
                play_sound_file(path);
                return builder;
            }
            return builder.sound(path);
        }
    }
    builder
}

pub fn send_reminder_notification(
    app: &AppHandle,
    title: &str,
    body: &str,
    reminder_id: &str,
    sound_path: Option<&str>,
) {
    if let Ok(state) = app.notification().permission_state() {
        if state != PermissionState::Granted {
            let _ = app.notification().request_permission();
        }
    }
    let _ = add_sound(
        app.notification()
            .builder()
            .title(title)
            .body(body)
            .extra("reminder_id", reminder_id),
        sound_path,
    )
    .show();
}

pub fn send_test_notification(app: &AppHandle, sound_path: Option<&str>) {
    let _ = add_sound(
        app.notification()
            .builder()
            .title("Recordatorio")
            .body("Notificación de prueba"),
        sound_path,
    )
    .show();
}
