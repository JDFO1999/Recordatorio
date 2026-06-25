
use rusqlite::{Connection, params, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tiberius::{Client, Config, EncryptionLevel, Query};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

type RemoteClient = Client<Compat<TcpStream>>;

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
    pub repeat_interval_seconds: Option<i64>,
    pub created_by: Option<String>,
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
    conn: Mutex<Connection>,
    remote_config: Option<Config>,
    remote_ensured: AtomicBool,
    use_remote: AtomicBool,
}

fn device_name() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string())
}

impl Database {
    pub fn new(app_dir: PathBuf, conn_string: Option<String>) -> Result<Self, String> {
        std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
        let db_path = app_dir.join("recordatorio.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

        let remote_config = match conn_string {
            Some(ref s) => Config::from_ado_string(s).ok().map(|mut c| {
                c.encryption(EncryptionLevel::NotSupported);
                c
            }),
            None => None,
        };

        let db = Database {
            conn: Mutex::new(conn),
            remote_config,
            remote_ensured: AtomicBool::new(false),
            use_remote: AtomicBool::new(false),
        };
        db.initialize_local_tables().map_err(|e| e.to_string())?;
        db.insert_default_settings().map_err(|e| e.to_string())?;
        db.insert_default_shortcuts().map_err(|e| e.to_string())?;
        // Load persisted db_mode
        if let Ok(Some(mode)) = db.get_setting("db_mode") {
            if mode == "compartido" && db.remote_config.is_some() {
                db.use_remote.store(true, Ordering::Relaxed);
            }
        }
        Ok(db)
    }

    // --- Local SQLite initialization ---

    fn initialize_local_tables(&self) -> SqliteResult<()> {
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
                source TEXT DEFAULT 'voice',
                repeat_interval_seconds INTEGER,
                created_by TEXT
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
        conn.execute_batch("ALTER TABLE reminders ADD COLUMN repeat_interval_seconds INTEGER;").ok();
        conn.execute_batch("ALTER TABLE reminders ADD COLUMN created_by TEXT;").ok();
        Ok(())
    }

    fn insert_default_settings(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let defaults = vec![
            ("autostart", "true"),
            ("scheduler_interval_secs", "30"),
            ("notification_sound_path", ""),
            ("notify_before_minutes", "2"),
            ("theme", "light"),
            ("transcription_provider", "whisper"),
            ("db_mode", "local"),
        ];
        for (key, value) in defaults {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                params![key, value, now],
            ).map_err(|e| e)?;
        }
        Ok(())
    }

    fn insert_default_shortcuts(&self) -> SqliteResult<()> {
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
            ).map_err(|e| e)?;
        }
        Ok(())
    }

    // --- Remote connection ---

    async fn get_remote_client(&self) -> Result<RemoteClient, String> {
        if !self.remote_ensured.load(Ordering::Relaxed) {
            self.ensure_remote_tables().await?;
            self.remote_ensured.store(true, Ordering::Relaxed);
        }
        let config = self.remote_config.as_ref().ok_or_else(|| "SQL Server no configurado".to_string())?;
        let addr = config.get_addr();
        let tcp = TcpStream::connect(addr.as_str()).await.map_err(|e| format!("No se pudo conectar a SQL Server ({}): {}", addr, e))?;
        tcp.set_nodelay(true).map_err(|e| format!("Error en socket: {}", e))?;
        Client::connect(config.clone(), tcp.compat_write()).await.map_err(|e| format!("Error de autenticación con SQL Server: {}", e))
    }

    async fn ensure_remote_tables(&self) -> Result<(), String> {
        if self.remote_config.is_none() { return Ok(()); }
        let config = self.remote_config.as_ref().unwrap();

        let tcp = TcpStream::connect(config.get_addr().as_str()).await
            .map_err(|e| format!("No se pudo conectar a SQL Server ({}): {}", config.get_addr(), e))?;
        tcp.set_nodelay(true).ok();
        let mut client = Client::connect(config.clone(), tcp.compat_write()).await
            .map_err(|e| format!("Error de autenticación con SQL Server: {}", e))?;
        client.simple_query(
            "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'reminders')
             BEGIN
                 CREATE TABLE reminders (
                     id NVARCHAR(36) PRIMARY KEY,
                     title NVARCHAR(500) NOT NULL,
                     description NVARCHAR(MAX),
                     original_text NVARCHAR(MAX),
                     due_at NVARCHAR(30) NOT NULL,
                     status NVARCHAR(20) NOT NULL DEFAULT 'pending',
                     created_at NVARCHAR(30) NOT NULL,
                     updated_at NVARCHAR(30) NOT NULL,
                     notified_at NVARCHAR(30),
                     completed_at NVARCHAR(30),
                     cancelled_at NVARCHAR(30),
                     snooze_count INT DEFAULT 0,
                     last_snoozed_at NVARCHAR(30),
                     parsed_time_expression NVARCHAR(200),
                     source NVARCHAR(20) DEFAULT 'voice',
                     repeat_interval_seconds BIGINT,
                     created_by NVARCHAR(100) NOT NULL
                 );
             END"
        ).await.map_err(|e| format!("Error creando tabla remota: {}", e))?;
        Ok(())
    }

    fn reminder_from_row(row: &tiberius::Row) -> Option<Reminder> {
        let get_str = |col: &str| -> Option<String> {
            row.get::<&str, _>(col).map(|s| s.to_string())
        };
        Some(Reminder {
            id: row.get::<&str, _>("id")?.to_string(),
            title: row.get::<&str, _>("title")?.to_string(),
            description: get_str("description"),
            original_text: get_str("original_text"),
            due_at: row.get::<&str, _>("due_at")?.to_string(),
            status: row.get::<&str, _>("status")?.to_string(),
            created_at: row.get::<&str, _>("created_at")?.to_string(),
            updated_at: row.get::<&str, _>("updated_at")?.to_string(),
            notified_at: get_str("notified_at"),
            completed_at: get_str("completed_at"),
            cancelled_at: get_str("cancelled_at"),
            snooze_count: row.get::<i32, _>("snooze_count").unwrap_or(0),
            last_snoozed_at: get_str("last_snoozed_at"),
            parsed_time_expression: get_str("parsed_time_expression"),
            source: row.get::<&str, _>("source")?.to_string(),
            repeat_interval_seconds: row.get::<i64, _>("repeat_interval_seconds"),
            created_by: get_str("created_by"),
        })
    }

    // --- Reminder CRUD (remote if configured, else local) ---

    pub async fn create_reminder(&self, reminder: &Reminder) -> Result<(), String> {
        let device = device_name();
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            let mut client = self.get_remote_client().await?;
            let mut q = Query::new(
                "INSERT INTO reminders (id, title, description, original_text, due_at, status, created_at, updated_at, snooze_count, source, parsed_time_expression, repeat_interval_seconds, created_by)
                 VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11, @P12, @P13)"
            );
            q.bind(reminder.id.as_str());
            q.bind(reminder.title.as_str());
            q.bind(reminder.description.as_deref());
            q.bind(reminder.original_text.as_deref());
            q.bind(reminder.due_at.as_str());
            q.bind(reminder.status.as_str());
            q.bind(reminder.created_at.as_str());
            q.bind(reminder.updated_at.as_str());
            q.bind(reminder.snooze_count);
            q.bind(reminder.source.as_str());
            q.bind(reminder.parsed_time_expression.as_deref());
            q.bind(reminder.repeat_interval_seconds);
            q.bind(device.as_str());
            q.execute(&mut client).await.map_err(|e| format!("Error al crear recordatorio remoto: {}", e))?;
        } else {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO reminders (id, title, description, original_text, due_at, status, created_at, updated_at, snooze_count, source, parsed_time_expression, repeat_interval_seconds, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    reminder.id, reminder.title, reminder.description, reminder.original_text,
                    reminder.due_at, reminder.status, reminder.created_at, reminder.updated_at,
                    reminder.snooze_count, reminder.source, reminder.parsed_time_expression,
                    reminder.repeat_interval_seconds, device,
                ],
            ).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn get_pending_reminders(&self) -> Result<Vec<Reminder>, String> {
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            let mut client = self.get_remote_client().await?;
            let mut query = Query::new(
                "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                        notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                        parsed_time_expression, source, repeat_interval_seconds, created_by
                 FROM reminders WHERE status IN (@P1, @P2) ORDER BY due_at ASC"
            );
            query.bind("pending");
            query.bind("notified");
            let stream = query.query(&mut client).await.map_err(|e| format!("Error consultando reminders: {}", e))?;
            let rows = stream.into_first_result().await.map_err(|e| format!("Error obteniendo filas: {}", e))?;
            let reminders: Vec<Reminder> = rows.iter().filter_map(|row| Self::reminder_from_row(row)).collect();
            Ok(reminders)
        } else {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                        notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                        parsed_time_expression, source, repeat_interval_seconds, created_by
                 FROM reminders WHERE status IN ('pending', 'notified') ORDER BY due_at ASC"
            ).map_err(|e| e.to_string())?;
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
                    repeat_interval_seconds: row.get(15)?,
                    created_by: row.get(16)?,
                })
            }).map_err(|e| e.to_string())?;
            let mut reminders = Vec::new();
            for row in rows {
                reminders.push(row.map_err(|e| e.to_string())?);
            }
            Ok(reminders)
        }
    }

    pub async fn get_all_reminders(&self, status_filter: Option<&str>) -> Result<Vec<Reminder>, String> {
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            let mut client = self.get_remote_client().await?;
            let sql = match status_filter {
                Some(_) => "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                           notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                           parsed_time_expression, source, repeat_interval_seconds, created_by
                    FROM reminders WHERE status = @P1 ORDER BY due_at DESC",
                None => "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                           notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                           parsed_time_expression, source, repeat_interval_seconds, created_by
                    FROM reminders ORDER BY due_at DESC",
            };
            let mut query = Query::new(sql);
            if let Some(s) = status_filter {
                query.bind(s);
            }
            let stream = query.query(&mut client).await.map_err(|e| format!("Error consultando reminders: {}", e))?;
            let rows = stream.into_first_result().await.map_err(|e| format!("Error obteniendo filas: {}", e))?;
            let reminders: Vec<Reminder> = rows.iter().filter_map(|row| Self::reminder_from_row(row)).collect();
            Ok(reminders)
        } else {
            let conn = self.conn.lock().unwrap();
            let (query, status_param): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match status_filter {
                Some(s) => (
                    "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                            notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                            parsed_time_expression, source, repeat_interval_seconds, created_by
                     FROM reminders WHERE status = ?1 ORDER BY due_at DESC".to_string(),
                    vec![Box::new(s.to_string())],
                ),
                None => (
                    "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                            notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                            parsed_time_expression, source, repeat_interval_seconds, created_by
                     FROM reminders ORDER BY due_at DESC".to_string(),
                    vec![],
                ),
            };
            let params_refs: Vec<&dyn rusqlite::types::ToSql> = status_param.iter().map(|p| p.as_ref()).collect();
            let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
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
                    repeat_interval_seconds: row.get(15)?,
                    created_by: row.get(16)?,
                })
            }).map_err(|e| e.to_string())?;
            let mut reminders = Vec::new();
            for row in rows {
                reminders.push(row.map_err(|e| e.to_string())?);
            }
            Ok(reminders)
        }
    }

    pub async fn get_reminder_by_id(&self, id: &str) -> Result<Option<Reminder>, String> {
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            let mut client = self.get_remote_client().await?;
            let mut query = Query::new(
                "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                        notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                        parsed_time_expression, source, repeat_interval_seconds, created_by
                 FROM reminders WHERE id = @P1"
            );
            query.bind(id);
            let stream = query.query(&mut client).await.map_err(|e| format!("Error consultando reminder: {}", e))?;
            let row = stream.into_row().await.map_err(|e| format!("Error obteniendo fila: {}", e))?;
            match row {
                Some(r) => Ok(Self::reminder_from_row(&r)),
                None => Ok(None),
            }
        } else {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                        notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                        parsed_time_expression, source, repeat_interval_seconds, created_by
                 FROM reminders WHERE id = ?1"
            ).map_err(|e| e.to_string())?;
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
                    repeat_interval_seconds: row.get(15)?,
                    created_by: row.get(16)?,
                })
            }).map_err(|e| e.to_string())?;
            match rows.next() {
                Some(Ok(r)) => Ok(Some(r)),
                _ => Ok(None),
            }
        }
    }

    pub async fn update_reminder(&self, id: &str, title: &str, description: Option<&str>, due_at: &str) -> Result<(), String> {
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            let mut client = self.get_remote_client().await?;
            let mut q = Query::new("UPDATE reminders SET title = @P1, description = @P2, due_at = @P3, updated_at = @P4 WHERE id = @P5");
            q.bind(title);
            q.bind(description);
            q.bind(due_at);
            q.bind(now.as_str());
            q.bind(id);
            q.execute(&mut client).await.map_err(|e| format!("Error actualizando reminder: {}", e))?;
        } else {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "UPDATE reminders SET title = ?1, description = ?2, due_at = ?3, updated_at = ?4 WHERE id = ?5",
                params![title, description, due_at, now, id],
            ).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn update_reminder_status(&self, id: &str, status: &str) -> Result<(), String> {
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let (status_col, status_val) = match status {
            "notified" => ("notified_at", Some(now.clone())),
            "completed" => ("completed_at", Some(now.clone())),
            "cancelled" => ("cancelled_at", Some(now.clone())),
            _ => ("updated_at", None),
        };
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            let mut client = self.get_remote_client().await?;
            match status_val {
                Some(val) => {
                    let sql = format!("UPDATE reminders SET status = @P1, updated_at = @P2, {} = @P3 WHERE id = @P4", status_col);
                    let mut q = Query::new(&sql);
                    q.bind(status);
                    q.bind(now.as_str());
                    q.bind(val.as_str());
                    q.bind(id);
                    q.execute(&mut client).await.map_err(|e| format!("Error actualizando estado: {}", e))?;
                }
                None => {
                    let mut q = Query::new("UPDATE reminders SET status = @P1, updated_at = @P2 WHERE id = @P3");
                    q.bind(status);
                    q.bind(now.as_str());
                    q.bind(id);
                    q.execute(&mut client).await.map_err(|e| format!("Error actualizando estado: {}", e))?;
                }
            }
        } else {
            let conn = self.conn.lock().unwrap();
            match status_val {
                Some(val) => {
                    conn.execute(
                        &format!("UPDATE reminders SET status = ?1, updated_at = ?2, {} = ?3 WHERE id = ?4", status_col),
                        params![status, now, val, id],
                    ).map_err(|e| e.to_string())?;
                }
                None => {
                    conn.execute(
                        "UPDATE reminders SET status = ?1, updated_at = ?2 WHERE id = ?3",
                        params![status, now, id],
                    ).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    pub async fn snooze_reminder(&self, id: &str, snooze_minutes: i32) -> Result<(), String> {
        let now = chrono::Local::now();
        let new_due = now + chrono::Duration::minutes(snooze_minutes as i64);
        let new_due_str = new_due.format("%Y-%m-%dT%H:%M:%S").to_string();
        let now_str = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            let mut client = self.get_remote_client().await?;
            let mut q = Query::new(
                "UPDATE reminders SET status = @P1, due_at = @P2, snooze_count = snooze_count + 1, last_snoozed_at = @P3, updated_at = @P3 WHERE id = @P4"
            );
            q.bind("pending");
            q.bind(new_due_str.as_str());
            q.bind(now_str.as_str());
            q.bind(id);
            q.execute(&mut client).await.map_err(|e| format!("Error al posponer: {}", e))?;
        } else {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "UPDATE reminders SET status = 'pending', due_at = ?1, snooze_count = snooze_count + 1,
                 last_snoozed_at = ?2, updated_at = ?2 WHERE id = ?3",
                params![new_due_str, now_str, id],
            ).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn reschedule_repeating(&self, id: &str, new_due_at: &str) -> Result<(), String> {
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            let mut client = self.get_remote_client().await?;
            let mut q = Query::new("UPDATE reminders SET due_at = @P1, updated_at = @P2, status = @P3 WHERE id = @P4");
            q.bind(new_due_at);
            q.bind(now.as_str());
            q.bind("pending");
            q.bind(id);
            q.execute(&mut client).await.map_err(|e| format!("Error reprogramando: {}", e))?;
        } else {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "UPDATE reminders SET due_at = ?1, updated_at = ?2, status = 'pending' WHERE id = ?3",
                params![new_due_at, now, id],
            ).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn delete_reminder(&self, id: &str) -> Result<(), String> {
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            let mut client = self.get_remote_client().await?;
            client.simple_query(&format!("DELETE FROM reminders WHERE id = '{}'", id))
                .await.map_err(|e| format!("Error eliminando: {}", e))?;
        } else {
            let conn = self.conn.lock().unwrap();
            conn.execute("DELETE FROM notification_events WHERE reminder_id = ?1", params![id]).map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM reminders WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    // --- DB mode toggle ---

    pub fn get_db_mode(&self) -> String {
        if self.use_remote.load(Ordering::Relaxed) && self.remote_config.is_some() {
            "compartido".to_string()
        } else {
            "local".to_string()
        }
    }

    pub fn set_db_mode(&self, mode: &str) -> Result<(), String> {
        match mode {
            "local" => {
                self.use_remote.store(false, Ordering::Relaxed);
                self.set_setting("db_mode", "local")?;
            }
            "compartido" => {
                if self.remote_config.is_none() {
                    return Err("No hay configuración de SQL Server disponible".to_string());
                }
                self.use_remote.store(true, Ordering::Relaxed);
                self.set_setting("db_mode", "compartido")?;
            }
            _ => return Err(format!("Modo inválido: {}", mode)),
        }
        Ok(())
    }

    // --- Settings (always local) ---

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1").map_err(|e| e.to_string())?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0)).map_err(|e| e.to_string())?;
        match rows.next() {
            Some(Ok(v)) => Ok(Some(v)),
            _ => Ok(None),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, now],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<Vec<AppSetting>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value, updated_at FROM settings ORDER BY key").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        }).map_err(|e| e.to_string())?;
        let mut settings = Vec::new();
        for row in rows {
            settings.push(row.map_err(|e| e.to_string())?);
        }
        Ok(settings)
    }

    // --- Shortcuts (always local) ---

    pub fn get_all_shortcuts(&self) -> Result<Vec<Shortcut>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, action, accelerator, enabled, created_at, updated_at FROM shortcuts ORDER BY action"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(Shortcut {
                id: row.get(0)?,
                action: row.get(1)?,
                accelerator: row.get(2)?,
                enabled: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;
        let mut shortcuts = Vec::new();
        for row in rows {
            shortcuts.push(row.map_err(|e| e.to_string())?);
        }
        Ok(shortcuts)
    }

    pub fn update_shortcut(&self, id: &str, accelerator: &str, enabled: bool) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        conn.execute(
            "UPDATE shortcuts SET accelerator = ?1, enabled = ?2, updated_at = ?3 WHERE id = ?4",
            params![accelerator, enabled as i32, now, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- Notification Events (always local) ---

    pub fn log_notification_event(&self, reminder_id: &str, event_type: &str, metadata: Option<&str>) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO notification_events (id, reminder_id, event_type, created_at, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, reminder_id, event_type, now, metadata],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_notification_events(&self, reminder_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM notification_events WHERE reminder_id = ?1", params![reminder_id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_notification_events_for_reminder(&self, reminder_id: &str, event_type: &str) -> Result<u32, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM notification_events WHERE reminder_id = ?1 AND event_type = ?2"
        ).map_err(|e| e.to_string())?;
        let count: u32 = stmt.query_row(params![reminder_id, event_type], |row| row.get(0)).map_err(|e| e.to_string())?;
        Ok(count)
    }

    #[allow(dead_code)]
    pub fn get_last_notified_reminder(&self) -> Result<Option<Reminder>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, original_text, due_at, status, created_at, updated_at,
                    notified_at, completed_at, cancelled_at, snooze_count, last_snoozed_at,
                    parsed_time_expression, source, repeat_interval_seconds, created_by
             FROM reminders WHERE status = 'notified' ORDER BY updated_at DESC LIMIT 1"
        ).map_err(|e| e.to_string())?;
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
                repeat_interval_seconds: row.get(15)?,
                created_by: row.get(16)?,
            })
        }).map_err(|e| e.to_string())?;
        match rows.next() {
            Some(Ok(r)) => Ok(Some(r)),
            _ => Ok(None),
        }
    }
}
