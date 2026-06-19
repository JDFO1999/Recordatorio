use crate::db::Database;
use crate::notifications;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tokio;

pub fn start_scheduler(app: AppHandle, db: Arc<Database>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let interval = get_scheduler_interval(&db);
            tokio::time::sleep(Duration::from_secs(interval)).await;
            check_and_notify(&app, &db).await;
        }
    });
}

fn get_scheduler_interval(db: &Database) -> u64 {
    match db.get_setting("scheduler_interval_secs") {
        Ok(Some(val)) => val.parse::<u64>().unwrap_or(30),
        _ => 30,
    }
}

async fn check_and_notify(app: &AppHandle, db: &Database) {
    if let Ok(reminders) = db.get_pending_reminders() {
        let now = chrono::Local::now();
        for reminder in reminders {
            if reminder.status != "pending" {
                continue;
            }
            if let Ok(due) =
                chrono::NaiveDateTime::parse_from_str(&reminder.due_at, "%Y-%m-%dT%H:%M:%S")
            {
                let due_local = due.and_local_timezone(chrono::Local).unwrap();
                if due_local <= now {
                    let already_notified = db
                        .get_notification_events_for_reminder(&reminder.id, "displayed")
                        .unwrap_or(0)
                        > 0;
                    if already_notified {
                        continue;
                    }

                    let sound_path = db
                        .get_setting("notification_sound_path")
                        .unwrap_or(None)
                        .unwrap_or_default();
                    let sound_path = if sound_path.is_empty() { None } else { Some(sound_path) };
                    notifications::send_reminder_notification(
                        app,
                        &reminder.title,
                        &format!("🔔 {}", reminder.title),
                        &reminder.id,
                        sound_path.as_deref(),
                    );
                    if let Err(e) = db.update_reminder_status(&reminder.id, "notified") {
                        eprintln!("Failed to update reminder status: {}", e);
                    }
                    if let Err(e) =
                        db.log_notification_event(&reminder.id, "displayed", None)
                    {
                        eprintln!("Failed to log notification event: {}", e);
                    }
                }
            }
        }
    }
}

pub fn check_overdue_on_startup(db: &Database, app: &AppHandle) {
    if let Ok(reminders) = db.get_pending_reminders() {
        let now = chrono::Local::now();
        for reminder in reminders {
            if reminder.status != "pending" {
                continue;
            }
            if let Ok(due) =
                chrono::NaiveDateTime::parse_from_str(&reminder.due_at, "%Y-%m-%dT%H:%M:%S")
            {
                let due_local = due.and_local_timezone(chrono::Local).unwrap();
                if due_local <= now {
                    if reminder.status == "notified" {
                        continue;
                    }
                    let sound_path = db
                        .get_setting("notification_sound_path")
                        .unwrap_or(None)
                        .unwrap_or_default();
                    let sound_path = if sound_path.is_empty() { None } else { Some(sound_path) };
                    notifications::send_reminder_notification(
                        app,
                        &format!("📋 Vencido: {}", reminder.title),
                        &format!(
                            "Este recordatorio venció a las {}",
                            reminder.due_at
                        ),
                        &reminder.id,
                        sound_path.as_deref(),
                    );
                    if let Err(e) = db.update_reminder_status(&reminder.id, "notified") {
                        eprintln!("Failed to update reminder status: {}", e);
                    }
                }
            }
        }
    }
}
