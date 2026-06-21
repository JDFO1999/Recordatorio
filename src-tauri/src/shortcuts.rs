use crate::db::Database;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut as TauriShortcut, ShortcutState,
};

pub fn register_all(app: &AppHandle, db: &Arc<Database>) {
    let shortcuts = match db.get_all_shortcuts() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load shortcuts from DB: {}", e);
            return;
        }
    };

    for sc in shortcuts {
        if !sc.enabled {
            continue;
        }
        match register_single_with_handler(app, &sc.action, &sc.accelerator, Arc::clone(db)) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "Failed to register shortcut '{}' ({}): {}",
                    sc.action, sc.accelerator, e
                );
            }
        }
    }
}

pub fn register_single_with_handler(
    app: &AppHandle,
    action: &str,
    accelerator: &str,
    db: Arc<Database>,
) -> Result<(), String> {
    let shortcut = parse_accelerator(accelerator)?;
    let action_owned = action.to_string();

    app.global_shortcut()
        .on_shortcut(shortcut, move |h, _s, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }

            let window = h.get_webview_window("main");

            match action_owned.as_str() {
                "start_recording" => {
                    let _ = h.emit("global-shortcut:start-recording", ());
                }
                "toggle_window" => {
                    if let Some(w) = window {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                }
                "new_reminder" => {
                    let _ = h.emit("global-shortcut:new-reminder", ());
                    if let Some(w) = window {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "snooze_last" => {
                    if let Ok(reminders) = db.get_pending_reminders() {
                        if let Some(last) = reminders.into_iter().last() {
                            let _ = db.snooze_reminder(&last.id, 5);
                            let _ = db.log_notification_event(
                                &last.id,
                                "snoozed",
                                Some(r#"{"source":"shortcut"}"#),
                            );
                            let _ = h.emit("global-shortcut:snoozed", last.id);
                        }
                    }
                }
                "complete_last" => {
                    if let Ok(reminders) = db.get_pending_reminders() {
                        if let Some(last) = reminders.into_iter().last() {
                            let _ = db.update_reminder_status(&last.id, "completed");
                            let _ = db.log_notification_event(
                                &last.id,
                                "completed",
                                Some(r#"{"source":"shortcut"}"#),
                            );
                            let _ = h.emit("global-shortcut:completed", last.id);
                        }
                    }
                }
                _ => {}
            }
        })
        .map_err(|e| format!("Failed to register shortcut '{}': {}", accelerator, e))
}

#[allow(dead_code)]
pub fn register_single(app: &AppHandle, accelerator: &str) -> Result<(), String> {
    let shortcut = parse_accelerator(accelerator)?;
    app.global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Failed to register shortcut '{}': {}", accelerator, e))
}

pub fn unregister_all(app: &AppHandle) {
    let _ = app.global_shortcut().unregister_all();
}

#[allow(dead_code)]
pub fn unregister(app: &AppHandle, accelerator: &str) -> Result<(), String> {
    let shortcut = parse_accelerator(accelerator)?;
    app.global_shortcut()
        .unregister(shortcut)
        .map_err(|e| format!("Failed to unregister: {}", e))
}

#[allow(dead_code)]
pub fn is_registered(app: &AppHandle, accelerator: &str) -> Result<bool, String> {
    let shortcut = parse_accelerator(accelerator)?;
    Ok(app.global_shortcut().is_registered(shortcut))
}

fn parse_accelerator(accel: &str) -> Result<TauriShortcut, String> {
    let parts: Vec<&str> = accel.split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut code: Option<Code> = None;

    for part in parts {
        let part = part.trim().to_lowercase();
        match part.as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "win" | "cmd" | "meta" => modifiers |= Modifiers::SUPER,
            other => {
                code = Some(match other {
                    "a" => Code::KeyA,
                    "b" => Code::KeyB,
                    "c" => Code::KeyC,
                    "d" => Code::KeyD,
                    "e" => Code::KeyE,
                    "f" => Code::KeyF,
                    "g" => Code::KeyG,
                    "h" => Code::KeyH,
                    "i" => Code::KeyI,
                    "j" => Code::KeyJ,
                    "k" => Code::KeyK,
                    "l" => Code::KeyL,
                    "m" => Code::KeyM,
                    "n" => Code::KeyN,
                    "o" => Code::KeyO,
                    "p" => Code::KeyP,
                    "q" => Code::KeyQ,
                    "r" => Code::KeyR,
                    "s" => Code::KeyS,
                    "t" => Code::KeyT,
                    "u" => Code::KeyU,
                    "v" => Code::KeyV,
                    "w" => Code::KeyW,
                    "x" => Code::KeyX,
                    "y" => Code::KeyY,
                    "z" => Code::KeyZ,
                    "f1" => Code::F1,
                    "f2" => Code::F2,
                    "f3" => Code::F3,
                    "f4" => Code::F4,
                    "f5" => Code::F5,
                    "f6" => Code::F6,
                    "f7" => Code::F7,
                    "f8" => Code::F8,
                    "f9" => Code::F9,
                    "f10" => Code::F10,
                    "f11" => Code::F11,
                    "f12" => Code::F12,
                    "space" => Code::Space,
                    "enter" | "return" => Code::Enter,
                    "escape" | "esc" => Code::Escape,
                    "tab" => Code::Tab,
                    "delete" | "del" => Code::Delete,
                    "backspace" => Code::Backspace,
                    "up" => Code::ArrowUp,
                    "down" => Code::ArrowDown,
                    "left" => Code::ArrowLeft,
                    "right" => Code::ArrowRight,
                    _ => return Err(format!("Unknown key: {}", other)),
                });
            }
        }
    }

    let key_code = code.ok_or("No key specified in shortcut")?;
    Ok(TauriShortcut::new(Some(modifiers), key_code))
}
