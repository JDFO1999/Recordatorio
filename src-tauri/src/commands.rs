use crate::date_parser;
use crate::db::{AppSetting, Database, Reminder, Shortcut};
use crate::notifications;
use crate::shortcuts;
use crate::whisper::{ModelInfo, ModelStatus, WhisperEngine};
use std::sync::Arc;
use tauri::AppHandle;
use tauri::State;

#[tauri::command]
pub async fn create_reminder(
    db: State<'_, Arc<Database>>,
    title: String,
    description: Option<String>,
    due_at: String,
    source: Option<String>,
    repeat_interval_seconds: Option<i64>,
) -> Result<Reminder, String> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let device = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string());
    let reminder = Reminder {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        description,
        original_text: None,
        due_at,
        status: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
        notified_at: None,
        completed_at: None,
        cancelled_at: None,
        snooze_count: 0,
        last_snoozed_at: None,
        parsed_time_expression: None,
        source: source.unwrap_or_else(|| "manual".to_string()),
        repeat_interval_seconds,
        created_by: Some(device),
    };
    db.create_reminder(&reminder).await?;
    Ok(reminder)
}

#[tauri::command]
pub async fn create_reminder_from_voice(
    db: State<'_, Arc<Database>>,
    text: String,
) -> Result<Reminder, String> {
    let parsed = date_parser::parse_reminder_text(&text);
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let device = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string());
    let reminder = Reminder {
        id: uuid::Uuid::new_v4().to_string(),
        title: parsed.title,
        description: None,
        original_text: Some(parsed.original_text),
        due_at: parsed.due_at,
        status: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
        notified_at: None,
        completed_at: None,
        cancelled_at: None,
        snooze_count: 0,
        last_snoozed_at: None,
        parsed_time_expression: parsed.parsed_time_expression,
        source: "voice".to_string(),
        repeat_interval_seconds: parsed.repeat_interval_seconds,
        created_by: Some(device),
    };
    db.create_reminder(&reminder).await?;
    Ok(reminder)
}

#[tauri::command]
pub async fn get_pending_reminders(db: State<'_, Arc<Database>>) -> Result<Vec<Reminder>, String> {
    db.get_pending_reminders().await
}

#[tauri::command]
pub async fn get_pending_reminders_count(
    db: State<'_, Arc<Database>>,
) -> Result<usize, String> {
    let reminders = db.get_pending_reminders().await?;
    Ok(reminders.len())
}

#[tauri::command]
pub async fn get_all_reminders(
    db: State<'_, Arc<Database>>,
    status_filter: Option<String>,
) -> Result<Vec<Reminder>, String> {
    db.get_all_reminders(status_filter.as_deref()).await
}

#[tauri::command]
pub async fn get_reminder_by_id(
    db: State<'_, Arc<Database>>,
    id: String,
) -> Result<Option<Reminder>, String> {
    db.get_reminder_by_id(&id).await
}

#[tauri::command]
pub async fn update_reminder(
    db: State<'_, Arc<Database>>,
    id: String,
    title: String,
    description: Option<String>,
    due_at: String,
) -> Result<(), String> {
    db.update_reminder(&id, &title, description.as_deref(), &due_at).await
}

#[tauri::command]
pub async fn mark_completed(
    db: State<'_, Arc<Database>>,
    id: String,
) -> Result<(), String> {
    db.update_reminder_status(&id, "completed").await?;
    db.log_notification_event(&id, "completed", None)
}

#[tauri::command]
pub async fn mark_cancelled(
    db: State<'_, Arc<Database>>,
    id: String,
) -> Result<(), String> {
    db.update_reminder_status(&id, "cancelled").await?;
    db.log_notification_event(&id, "cancelled", None)
}

#[tauri::command]
pub async fn snooze_reminder(
    db: State<'_, Arc<Database>>,
    id: String,
    minutes: i32,
) -> Result<(), String> {
    db.snooze_reminder(&id, minutes).await?;
    db.log_notification_event(
        &id,
        "snoozed",
        Some(&format!("{{\"minutes\":{}}}", minutes)),
    )
}

#[tauri::command]
pub async fn delete_reminder(
    db: State<'_, Arc<Database>>,
    id: String,
) -> Result<(), String> {
    db.delete_reminder(&id).await
}

#[tauri::command]
pub fn parse_text(text: String) -> Result<date_parser::ParsedReminder, String> {
    Ok(date_parser::parse_reminder_text(&text))
}

#[tauri::command]
pub fn get_settings(db: State<Arc<Database>>) -> Result<Vec<AppSetting>, String> {
    db.get_all_settings()
}

#[tauri::command]
pub fn get_setting(db: State<Arc<Database>>, key: String) -> Result<Option<String>, String> {
    db.get_setting(&key)
}

#[tauri::command]
pub fn set_setting(db: State<Arc<Database>>, key: String, value: String) -> Result<(), String> {
    db.set_setting(&key, &value)
}

#[tauri::command]
pub fn get_shortcuts(db: State<Arc<Database>>) -> Result<Vec<Shortcut>, String> {
    db.get_all_shortcuts()
}

#[tauri::command]
pub fn update_shortcut(
    db: State<Arc<Database>>,
    app: AppHandle,
    id: String,
    accelerator: String,
    enabled: bool,
) -> Result<(), String> {
    db.update_shortcut(&id, &accelerator, enabled)?;
    shortcuts::unregister_all(&app);
    shortcuts::register_all(&app, &db);
    Ok(())
}

#[tauri::command]
pub fn test_notification(app: AppHandle, db: State<Arc<Database>>) -> Result<(), String> {
    let sound_path = db
        .get_setting("notification_sound_path")
        .unwrap_or(None)
        .unwrap_or_default();
    let sound_path = if sound_path.is_empty() {
        None
    } else {
        Some(sound_path)
    };
    notifications::send_test_notification(&app, sound_path.as_deref(), true);
    Ok(())
}

#[tauri::command]
pub fn get_model_info(whisper: State<Arc<WhisperEngine>>) -> Result<ModelInfo, String> {
    Ok(whisper.get_model_info())
}

#[tauri::command]
pub async fn transcribe_audio(
    whisper: State<'_, Arc<WhisperEngine>>,
    path: String,
) -> Result<String, String> {
    let engine = Arc::clone(&whisper);
    tokio::task::spawn_blocking(move || engine.transcribe(&path))
        .await
        .map_err(|e| format!("Error en transcripción: {}", e))?
}

#[tauri::command]
pub fn get_model_status(whisper: State<Arc<WhisperEngine>>) -> Result<ModelStatus, String> {
    Ok(whisper.get_status())
}

#[tauri::command]
pub fn download_model(
    whisper: State<Arc<WhisperEngine>>,
    variant: Option<String>,
) -> Result<(), String> {
    let v = variant.as_deref().unwrap_or("small");
    whisper.start_download(v)
}

#[tauri::command]
pub fn refresh_shortcuts(db: State<Arc<Database>>, app: AppHandle) -> Result<(), String> {
    shortcuts::unregister_all(&app);
    shortcuts::register_all(&app, &db);
    Ok(())
}

#[tauri::command]
pub fn check_shortcut_conflict(accelerator: String) -> Result<bool, String> {
    let parts: Vec<&str> = accelerator.split('+').collect();
    if parts.len() < 2 {
        return Err("Debe incluir al menos una tecla modificadora (Ctrl, Alt, Shift)".to_string());
    }
    Ok(false)
}

#[tauri::command]
pub async fn snooze_last_reminder(
    db: State<'_, Arc<Database>>,
    minutes: i32,
) -> Result<(), String> {
    if let Ok(reminders) = db.get_pending_reminders().await {
        if let Some(last) = reminders.into_iter().last() {
            return db.snooze_reminder(&last.id, minutes).await;
        }
    }
    Err("No hay recordatorios pendientes".to_string())
}

#[tauri::command]
pub async fn complete_last_reminder(
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    if let Ok(reminders) = db.get_pending_reminders().await {
        if let Some(last) = reminders.into_iter().last() {
            db.update_reminder_status(&last.id, "completed").await?;
            return db.log_notification_event(&last.id, "completed", None);
        }
    }
    Err("No hay recordatorios pendientes".to_string())
}

#[tauri::command]
pub fn save_file(path: String, data: Vec<u8>) -> Result<(), String> {
    use std::io::Write;
    let parent = std::path::Path::new(&path)
        .parent()
        .ok_or("Invalid path")?;
    std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    let mut file =
        std::fs::File::create(&path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&data)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    if std::path::Path::new(&path).exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))?;
    }
    Ok(())
}
