//! Tauri shell. The WebView gets the minimum permission surface (ARCHITECTURE §7):
//! activity detection is exposed as exactly one narrow command returning resolved
//! data — the WebView can never enumerate processes or see raw identities.

use serde::Serialize;

/// What the status bar shows about activity detection. Resolved data only.
#[derive(Serialize)]
struct ActivityProbe {
    /// Which platform backend was selected, e.g. "linux-kwin". `"null"` means
    /// presence-only on this platform.
    backend: String,
    /// Resolved registry id of the current foreground app, if sharing is on and
    /// the app is in the registry. Never a raw process name, never a title.
    registry_id: Option<String>,
}

/// The one activity command (M5 wires real backends + the poller behind it).
#[tauri::command]
fn activity_probe() -> ActivityProbe {
    let (kind, _backend) = linger_activity::backend::select_backend();
    ActivityProbe {
        backend: format!("{kind:?}"),
        registry_id: None,
    }
}

/// Entry point shared by main.rs and (later) mobile.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![activity_probe])
        .run(tauri::generate_context!())
        .expect("failed to start linger");
}
