use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Runtime;
use tauri_plugin_notification::{NotificationExt, PermissionState};

#[link(name = "winmm")]
extern "system" {
    fn PlaySoundW(pszSound: *const u16, hmod: *mut std::ffi::c_void, fdwSound: u32);
}

const SND_FILENAME: u32 = 0x00020000;
const SND_ASYNC: u32 = 0x0001;
const SOUND_TIMEOUT_SECS: u64 = 20;

fn is_file_path(path: &str) -> bool {
    path.contains('\\') || path.contains('/') || path.ends_with(".wav")
}

fn play_sound_file(path: &str) {
    let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        PlaySoundW(wide.as_ptr(), null_mut(), SND_FILENAME | SND_ASYNC);
    }
    let _ = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(SOUND_TIMEOUT_SECS));
        unsafe {
            PlaySoundW(std::ptr::null(), null_mut(), 0);
        }
    });
}

fn play_and_wait_sound(path: &str) {
    let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        PlaySoundW(wide.as_ptr(), null_mut(), SND_FILENAME | SND_ASYNC);
    }
    std::thread::sleep(Duration::from_secs(SOUND_TIMEOUT_SECS));
    unsafe {
        PlaySoundW(std::ptr::null(), null_mut(), 0);
    }
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

pub fn send_test_notification(app: &AppHandle, sound_path: Option<&str>, wait: bool) {
    let sound_path: Option<&str> = sound_path.filter(|p| !p.is_empty());
    if let Some(path) = sound_path {
        if is_file_path(path) {
            if wait {
                play_and_wait_sound(path);
            } else {
                play_sound_file(path);
            }
            let _ = app.notification()
                .builder()
                .title("Recordatorio")
                .body("Notificación de prueba")
                .show();
            return;
        }
    }
    let builder = app.notification()
        .builder()
        .title("Recordatorio")
        .body("Notificación de prueba");
    let builder = if let Some(path) = sound_path {
        builder.sound(path)
    } else {
        builder
    };
    let _ = builder.show();
    if wait {
        std::thread::sleep(Duration::from_secs(SOUND_TIMEOUT_SECS));
    }
}
