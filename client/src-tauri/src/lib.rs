//! Tauri shell. The WebView gets the minimum permission surface (ARCHITECTURE §7):
//! activity detection is exposed as exactly one narrow command returning resolved
//! data — the WebView can never enumerate processes or see raw identities.

pub mod gateway;
mod secrets;

use std::collections::HashMap;
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

/// Save one server's session after a sign-in or a token refresh.
///
/// Other servers in the list are left alone. This server becomes the active
/// one, which is what a sign-in means.
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

/// Forget one server, or every server when `base_url` is omitted.
#[tauri::command]
async fn session_clear(base_url: Option<String>) -> SessionWrite {
    off_thread(
        move || secrets::clear(base_url.as_deref()),
        || SessionWrite::Unavailable {
            reason: lost_worker(),
        },
    )
    .await
}

/// Remember which server the window is looking at.
#[tauri::command]
async fn session_set_active(base_url: String) -> SessionWrite {
    off_thread(
        move || secrets::set_active(&base_url),
        || SessionWrite::Unavailable {
            reason: lost_worker(),
        },
    )
    .await
}

// ---------------------------------------------------------------------------
// Gateway
// ---------------------------------------------------------------------------

/// Live gateway connections, one per server. Killing or signing out of one
/// must not touch the others (T-412).
#[derive(Default)]
struct Connections(Mutex<HashMap<String, gateway::Handle>>);

impl Connections {
    fn with<T>(&self, work: impl FnOnce(&mut HashMap<String, gateway::Handle>) -> T) -> T {
        let mut map = match self.0.lock() {
            Ok(map) => map,
            // A panic elsewhere poisoned the lock. What it guards is still a
            // perfectly good map of handles, and refusing to reconnect for the
            // rest of the session would be the worse outcome by far.
            Err(poisoned) => poisoned.into_inner(),
        };
        work(&mut map)
    }
}

/// Status for one server. The frontend fans these out by `base_url`.
#[derive(Clone, Serialize)]
struct StatusEvent {
    base_url: String,
    status: gateway::Status,
}

/// A sequenced frame from one server.
#[derive(Clone, Serialize)]
struct FrameEvent {
    base_url: String,
    frame: ServerFrame,
}

/// Sends what the gateway client produces to the WebView, tagged with the
/// server it came from so two live connections cannot overwrite each other.
struct WindowEvents {
    app: AppHandle,
    base_url: String,
}

impl gateway::Events for WindowEvents {
    fn status(&self, status: gateway::Status) {
        // A failed emit means the window is gone. There is nobody to tell.
        let _ = self.app.emit(
            gateway::STATUS_EVENT,
            StatusEvent {
                base_url: self.base_url.clone(),
                status,
            },
        );
    }

    fn frame(&self, frame: &ServerFrame) {
        let _ = self.app.emit(
            gateway::FRAME_EVENT,
            FrameEvent {
                base_url: self.base_url.clone(),
                frame: frame.clone(),
            },
        );
    }
}

/// Open (or reopen) the connection to one server. Calling this again for the
/// same address replaces that connection only; every other server stays up.
///
/// `expires_at_ms` is when the access token dies, in Unix milliseconds; the
/// frontend knows it from the `expires_in` that came with the token. `false`
/// means the address was not one we can dial.
#[tauri::command]
fn gateway_connect(
    app: AppHandle,
    connections: State<'_, Connections>,
    base_url: String,
    token: String,
    expires_at_ms: i64,
) -> bool {
    let token = gateway::Token {
        value: token,
        expires_at_ms,
    };
    let Some((handle, task)) = gateway::client(
        &base_url,
        token,
        WindowEvents {
            app: app.clone(),
            base_url: base_url.clone(),
        },
    ) else {
        return false;
    };
    tauri::async_runtime::spawn(task);
    connections.with(|map| {
        if let Some(previous) = map.insert(base_url, handle) {
            previous.shutdown();
        }
    });
    true
}

/// Close one connection: sign-out of that server, or the frontend going away.
#[tauri::command]
fn gateway_disconnect(connections: State<'_, Connections>, base_url: String) {
    connections.with(|map| {
        if let Some(handle) = map.remove(&base_url) {
            handle.shutdown();
        }
    });
}

/// Hand one connection a fresh access token. The frontend is the only owner of
/// refresh tokens, so this is the only way a new one arrives.
#[tauri::command]
fn gateway_token(
    connections: State<'_, Connections>,
    base_url: String,
    token: String,
    expires_at_ms: i64,
) -> bool {
    connections.with(|map| {
        map.get(&base_url).is_some_and(|handle| {
            handle.set_token(gateway::Token {
                value: token,
                expires_at_ms,
            })
        })
    })
}

/// Send one client frame on one connection. `false` means there was no
/// connection to send it on.
#[tauri::command]
fn gateway_send(
    connections: State<'_, Connections>,
    base_url: String,
    frame: ClientFrame,
) -> bool {
    connections.with(|map| map.get(&base_url).is_some_and(|handle| handle.send(frame)))
}

/// Entry point shared by main.rs and (later) mobile.
pub fn run() {
    tauri::Builder::default()
        // Links in a message go to the system browser, never to this window.
        // The capability file narrows the plugin to http and https.
        .plugin(tauri_plugin_opener::init())
        // The one thing allowed to interrupt somebody: a message that names
        // them, or one from a person they asked to hear from (SPEC §4.2).
        // There are no other notifications and no unread badge to attach one to.
        .plugin(tauri_plugin_notification::init())
        .manage(Connections::default())
        .invoke_handler(tauri::generate_handler![
            activity_probe,
            session_load,
            session_save,
            session_clear,
            session_set_active,
            gateway_connect,
            gateway_disconnect,
            gateway_token,
            gateway_send
        ])
        .run(tauri::generate_context!())
        .expect("failed to start linger");
}
