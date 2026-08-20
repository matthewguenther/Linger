//! Tauri shell. The WebView gets the minimum permission surface (ARCHITECTURE §7):
//! activity detection is exposed as exactly one narrow command returning resolved
//! data — the WebView can never enumerate processes or see raw identities.

mod secrets;

use serde::Serialize;

use secrets::{SessionLoad, SessionWrite, StoredSession};

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

/// Keyring calls talk to a system daemon and can block for as long as it takes
/// the user to unlock a wallet, so they never run on a runtime thread. A task
/// that dies still has to produce an answer — the frontend must always get one.
async fn off_thread<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    on_lost: impl FnOnce() -> T,
) -> T {
    match tauri::async_runtime::spawn_blocking(work).await {
        Ok(value) => value,
        Err(_) => on_lost(),
    }
}

fn lost_worker() -> String {
    "The keyring lookup didn't finish.".to_string()
}

/// Read the saved session on startup. Never fails: "there is no keyring here"
/// comes back as `unavailable` so the app can ask for a fresh sign-in.
#[tauri::command]
async fn session_load() -> SessionLoad {
    off_thread(secrets::load, || SessionLoad::Unavailable {
        reason: lost_worker(),
    })
    .await
}

/// Save the session after a sign-in or a token refresh.
#[tauri::command]
async fn session_save(session: StoredSession) -> SessionWrite {
    off_thread(
        move || secrets::save(&session),
        || SessionWrite::Unavailable {
            reason: lost_worker(),
        },
    )
    .await
}

/// Forget the session on sign-out, or when the server rejects it.
#[tauri::command]
async fn session_clear() -> SessionWrite {
    off_thread(secrets::clear, || SessionWrite::Unavailable {
        reason: lost_worker(),
    })
    .await
}

/// Entry point shared by main.rs and (later) mobile.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            activity_probe,
            session_load,
            session_save,
            session_clear
        ])
        .run(tauri::generate_context!())
        .expect("failed to start linger");
}
