//! Tauri shell. The WebView gets the minimum permission surface (ARCHITECTURE §7):
//! activity detection is exposed as exactly one narrow command returning resolved
//! data — the WebView can never enumerate processes or see raw identities.

pub mod gateway;
mod secrets;

use std::sync::Mutex;

use linger_core::gateway::{ClientFrame, ServerFrame};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

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

// ---------------------------------------------------------------------------
// Gateway
// ---------------------------------------------------------------------------

/// The live gateway connection, if there is one. Replaced wholesale on sign-in
/// and torn down on sign-out: one connection to one server at a time.
#[derive(Default)]
struct Connection(Mutex<Option<gateway::Handle>>);

impl Connection {
    fn with<T>(&self, work: impl FnOnce(&mut Option<gateway::Handle>) -> T) -> T {
        let mut slot = match self.0.lock() {
            Ok(slot) => slot,
            // A panic elsewhere poisoned the lock. What it guards is still a
            // perfectly good handle, and refusing to reconnect for the rest of
            // the session would be the worse outcome by far.
            Err(poisoned) => poisoned.into_inner(),
        };
        work(&mut slot)
    }
}

/// Sends what the gateway client produces to the WebView.
struct WindowEvents {
    app: AppHandle,
}

impl gateway::Events for WindowEvents {
    fn status(&self, status: gateway::Status) {
        // A failed emit means the window is gone. There is nobody to tell.
        let _ = self.app.emit(gateway::STATUS_EVENT, status);
    }

    fn frame(&self, frame: &ServerFrame) {
        let _ = self.app.emit(gateway::FRAME_EVENT, frame);
    }
}

/// Open (or reopen) the connection. Calling this again replaces the previous
/// one, which is what makes a frontend reload or a re-sign-in start clean.
///
/// `expires_at_ms` is when the access token dies, in Unix milliseconds; the
/// frontend knows it from the `expires_in` that came with the token. `false`
/// means the address was not one we can dial.
#[tauri::command]
fn gateway_connect(
    app: AppHandle,
    connection: State<'_, Connection>,
    base_url: String,
    token: String,
    expires_at_ms: i64,
) -> bool {
    let token = gateway::Token {
        value: token,
        expires_at_ms,
    };
    let Some((handle, task)) = gateway::client(&base_url, token, WindowEvents { app: app.clone() })
    else {
        return false;
    };
    tauri::async_runtime::spawn(task);
    connection.with(|slot| {
        if let Some(previous) = slot.replace(handle) {
            previous.shutdown();
        }
    });
    true
}

/// Close the connection: sign-out, or the frontend going away.
#[tauri::command]
fn gateway_disconnect(connection: State<'_, Connection>) {
    connection.with(|slot| {
        if let Some(handle) = slot.take() {
            handle.shutdown();
        }
    });
}

/// Hand the connection a fresh access token. The frontend is the only owner of
/// refresh tokens, so this is the only way a new one arrives.
#[tauri::command]
fn gateway_token(connection: State<'_, Connection>, token: String, expires_at_ms: i64) -> bool {
    connection.with(|slot| {
        slot.as_ref().is_some_and(|handle| {
            handle.set_token(gateway::Token {
                value: token,
                expires_at_ms,
            })
        })
    })
}

/// Send one client frame. `false` means there was no connection to send it on.
#[tauri::command]
fn gateway_send(connection: State<'_, Connection>, frame: ClientFrame) -> bool {
    connection.with(|slot| slot.as_ref().is_some_and(|handle| handle.send(frame)))
}

/// Entry point shared by main.rs and (later) mobile.
pub fn run() {
    tauri::Builder::default()
        // Links in a message go to the system browser, never to this window.
        // The capability file narrows the plugin to http and https.
        .plugin(tauri_plugin_opener::init())
        .manage(Connection::default())
        .invoke_handler(tauri::generate_handler![
            activity_probe,
            session_load,
            session_save,
            session_clear,
            gateway_connect,
            gateway_disconnect,
            gateway_token,
            gateway_send
        ])
        .run(tauri::generate_context!())
        .expect("failed to start linger");
}
