use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reminder {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub original_text: Option<String>,
    pub due_at: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub notified_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub snooze_count: i32,
    pub last_snoozed_at: Option<String>,
    pub parsed_time_expression: Option<String>,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Shortcut {
    pub id: String,
    pub action: String,
    pub accelerator: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct NotificationEvent {
    pub id: String,
    pub reminder_id: String,
    pub event_type: String,
    pub created_at: String,
    pub metadata: Option<String>,
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("recordatorio.db");
        let conn = Connection::open(db_path)?;
        let db = Database { conn: Mutex::new(conn) };
        db.initialize_tables()?;
        db.insert_default_settings()?;
        db.insert_default_shortcuts()?;
        Ok(db)
    }

    fn initialize_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS reminders (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                original_text TEXT,
                due_at TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                notified_at TEXT,
                completed_at TEXT,
                cancelled_at TEXT,
                snooze_count INTEGER DEFAULT 0,
                last_snoozed_at TEXT,
                parsed_time_expression TEXT,
                source TEXT DEFAULT 'voice'
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS shortcuts (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL UNIQUE,
                accelerator TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS notification_events (
                id TEXT PRIMARY KEY,
                reminder_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                metadata TEXT,
                FOREIGN KEY (reminder_id) REFERENCES reminders(id)
            );

            CREATE INDEX IF NOT EXISTS idx_reminders_status ON reminders(status);
            CREATE INDEX IF NOT EXISTS idx_reminders_due_at ON reminders(due_at);
            CREATE INDEX IF NOT EXISTS idx_notification_events_reminder ON notification_events(reminder_id);"
        )?;
        Ok(())
    }

    fn insert_default_settings(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let defaults = vec![
            ("autostart", "true"),
            ("scheduler_interval_secs", "30"),
            ("notification_sound_path", ""),
            ("notify_before_minutes", "2"),
            ("theme", "light"),
            ("transcription_provider", "whisper"),
        ];
        for (key, value) in defaults {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                params![key, value, now],
            )?;
        }
        Ok(())
    }

    fn insert_default_shortcuts(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let defaults = vec![
            ("start_recording", "Ctrl+Alt+R"),
            ("toggle_window", "Ctrl+Alt+O"),
            ("new_reminder", "Ctrl+Alt+N"),
            ("snooze_last", "Ctrl+Alt+S"),
            ("complete_last", "Ctrl+Alt+D"),
        ];
        for (action, accelerator) in defaults {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT OR IGNORE INTO shortcuts (id, action, accelerator, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, 1, ?4, ?5)",
                params![id, action, accelerator, now, now],
            )?;
        }
        Ok(())
    }

    // Reminder CRUD
    pub fn create_reminder(&self, reminder: &Reminder) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO reminders (id, title, description, original_text, due_at, status, created_at, updated_at, snooze_count, source, parsed_time_expression)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                reminder.id, reminder.title, reminder.description, reminder.original_text,
                reminder.due_at, reminder.status, reminder.created_at, reminder.updated_at,
                reminder.snooze_count, reminder.source, reminder.parsed_time_expression
            ],
        )?;
        Ok(())
    }

    pub fn get_pending_reminders(&self) -> Result<Vec<Reminder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                    notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                    parsed_time_expression, source
             FROM reminders WHERE status IN ('pending', 'notified') ORDER BY due_at ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Reminder {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                original_text: row.get(3)?,
                due_at: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                notified_at: row.get(8)?,
                completed_at: row.get(9)?,
                cancelled_at: row.get(10)?,
                snooze_count: row.get(11)?,
                last_snoozed_at: row.get(12)?,
                parsed_time_expression: row.get(13)?,
                source: row.get(14)?,
            })
        })?;
        let mut reminders = Vec::new();
        for row in rows {
            reminders.push(row?);
        }
        Ok(reminders)
    }

    pub fn get_all_reminders(&self, status_filter: Option<&str>) -> Result<Vec<Reminder>> {
        let conn = self.conn.lock().unwrap();
        let (query, status_param): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match status_filter {
            Some(s) => (
                "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                        notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                        parsed_time_expression, source
                 FROM reminders WHERE status = ?1 ORDER BY due_at DESC".to_string(),
                vec![Box::new(s.to_string())],
            ),
            None => (
                "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                        notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                        parsed_time_expression, source
                 FROM reminders ORDER BY due_at DESC".to_string(),
                vec![],
            ),
        };
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = status_param.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Reminder {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                original_text: row.get(3)?,
                due_at: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                notified_at: row.get(8)?,
                completed_at: row.get(9)?,
                cancelled_at: row.get(10)?,
                snooze_count: row.get(11)?,
                last_snoozed_at: row.get(12)?,
                parsed_time_expression: row.get(13)?,
                source: row.get(14)?,
            })
        })?;
        let mut reminders = Vec::new();
        for row in rows {
            reminders.push(row?);
        }
        Ok(reminders)
    }

    pub fn get_reminder_by_id(&self, id: &str) -> Result<Option<Reminder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                    notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                    parsed_time_expression, source
             FROM reminders WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Reminder {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                original_text: row.get(3)?,
                due_at: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                notified_at: row.get(8)?,
                completed_at: row.get(9)?,
                cancelled_at: row.get(10)?,
                snooze_count: row.get(11)?,
                last_snoozed_at: row.get(12)?,
                parsed_time_expression: row.get(13)?,
                source: row.get(14)?,
            })
        })?;
        match rows.next() {
            Some(Ok(r)) => Ok(Some(r)),
            _ => Ok(None),
        }
    }

    pub fn update_reminder_status(&self, id: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let (status_col, status_val) = match status {
            "notified" => ("notified_at", Some(now.clone())),
            "completed" => ("completed_at", Some(now.clone())),
            "cancelled" => ("cancelled_at", Some(now.clone())),
            _ => ("updated_at", None),
        };
        match status_val {
            Some(val) => {
                conn.execute(
                    &format!("UPDATE reminders SET status = ?1, updated_at = ?2, {} = ?3 WHERE id = ?4", status_col),
                    params![status, now, val, id],
                )?;
            }
            None => {
                conn.execute(
                    "UPDATE reminders SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    params![status, now, id],
                )?;
            }
        }
        Ok(())
    }

    pub fn snooze_reminder(&self, id: &str, snooze_minutes: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now();
        let new_due = now + chrono::Duration::minutes(snooze_minutes as i64);
        let new_due_str = new_due.format("%Y-%m-%dT%H:%M:%S").to_string();
        let now_str = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        conn.execute(
            "UPDATE reminders SET status = 'pending', due_at = ?1, snooze_count = snooze_count + 1,
             last_snoozed_at = ?2, updated_at = ?2 WHERE id = ?3",
            params![new_due_str, now_str, id],
        )?;
        Ok(())
    }

    pub fn update_reminder(&self, id: &str, title: &str, description: Option<&str>, due_at: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        conn.execute(
            "UPDATE reminders SET title = ?1, description = ?2, due_at = ?3, updated_at = ?4 WHERE id = ?5",
            params![title, description, due_at, now, id],
        )?;
        Ok(())
    }

    pub fn delete_reminder(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM notification_events WHERE reminder_id = ?1", params![id])?;
        conn.execute("DELETE FROM reminders WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Settings
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(v)) => Ok(Some(v)),
            _ => Ok(None),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<Vec<AppSetting>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value, updated_at FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |row| {
            Ok(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })?;
        let mut settings = Vec::new();
        for row in rows {
            settings.push(row?);
        }
        Ok(settings)
    }

    // Shortcuts
    pub fn get_all_shortcuts(&self) -> Result<Vec<Shortcut>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, action, accelerator, enabled, created_at, updated_at FROM shortcuts ORDER BY action"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Shortcut {
                id: row.get(0)?,
                action: row.get(1)?,
                accelerator: row.get(2)?,
                enabled: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        let mut shortcuts = Vec::new();
        for row in rows {
            shortcuts.push(row?);
        }
        Ok(shortcuts)
    }

    pub fn update_shortcut(&self, id: &str, accelerator: &str, enabled: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        conn.execute(
            "UPDATE shortcuts SET accelerator = ?1, enabled = ?2, updated_at = ?3 WHERE id = ?4",
            params![accelerator, enabled as i32, now, id],
        )?;
        Ok(())
    }

    // Notification Events
    pub fn log_notification_event(&self, reminder_id: &str, event_type: &str, metadata: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO notification_events (id, reminder_id, event_type, created_at, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, reminder_id, event_type, now, metadata],
        )?;
        Ok(())
    }

    pub fn get_notification_events_for_reminder(&self, reminder_id: &str, event_type: &str) -> Result<u32> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM notification_events WHERE reminder_id = ?1 AND event_type = ?2"
        )?;
        let count: u32 = stmt.query_row(params![reminder_id, event_type], |row| row.get(0))?;
        Ok(count)
    }

    #[allow(dead_code)]
    pub fn get_last_notified_reminder(&self) -> Result<Option<Reminder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                    notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                    parsed_time_expression, source
             FROM reminders WHERE status = 'notified' ORDER BY updated_at DESC LIMIT 1"
        )?;
        let mut rows = stmt.query_map([], |row| {
            Ok(Reminder {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                original_text: row.get(3)?,
                due_at: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                notified_at: row.get(8)?,
                completed_at: row.get(9)?,
                cancelled_at: row.get(10)?,
                snooze_count: row.get(11)?,
                last_snoozed_at: row.get(12)?,
                parsed_time_expression: row.get(13)?,
                source: row.get(14)?,
            })
        })?;
        match rows.next() {
            Some(Ok(r)) => Ok(Some(r)),
            _ => Ok(None),
        }
    }
}
