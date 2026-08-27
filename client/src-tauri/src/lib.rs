//! Tauri shell. The WebView gets the minimum permission surface (ARCHITECTURE §7):
//! activity detection is exposed as exactly one narrow command returning resolved
//! data — the WebView can never enumerate processes or see raw identities.

pub mod gateway;
mod secrets;
mod updates;

use std::collections::HashMap;
use std::sync::Mutex;

use linger_core::gateway::{ClientFrame, ServerFrame};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use secrets::{SessionWrite, SessionsLoad, StoredSession};

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

/// The one activity command (real backends and the poller land with T-911…T-917,
/// on the backburner — until then this always reports `None`).
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

/// Read every saved sign-in on startup, oldest server first. Never fails:
/// "there is no keyring here" comes back as `unavailable` so the app can ask
/// for a fresh sign-in.
#[tauri::command]
async fn sessions_load() -> SessionsLoad {
    off_thread(secrets::load, || SessionsLoad::Unavailable {
        reason: lost_worker(),
    })
    .await
}

/// Save one server's session after a sign-in or a token refresh. Saving a
/// server that is already stored replaces its token and leaves the rest alone.
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

/// Forget one server on sign-out, or when it rejects our token. The other
/// servers' sign-ins are untouched.
#[tauri::command]
async fn session_forget(base_url: String) -> SessionWrite {
    off_thread(
        move || secrets::forget(&base_url),
        || SessionWrite::Unavailable {
            reason: lost_worker(),
        },
    )
    .await
}

// ---------------------------------------------------------------------------
// Gateway
// ---------------------------------------------------------------------------

/// The live gateway connections, one per server, keyed by base URL.
///
/// T-412: the client can be signed into several servers at once, and each one
/// gets its own socket, its own resume state and its own backoff. One server
/// going down is one entry retrying — the others never notice.
#[derive(Default)]
struct Connections(Mutex<HashMap<String, gateway::Handle>>);

impl Connections {
    fn with<T>(&self, work: impl FnOnce(&mut HashMap<String, gateway::Handle>) -> T) -> T {
        let mut held = match self.0.lock() {
            Ok(held) => held,
            // A panic elsewhere poisoned the lock. What it guards is still a
            // perfectly good set of handles, and refusing to reconnect for the
            // rest of the session would be the worse outcome by far.
            Err(poisoned) => poisoned.into_inner(),
        };
        work(&mut held)
    }
}

/// A status change, tagged with the server it came from. Hand-written on both
/// sides and never on the wire, same as `Status` itself. `Clone` because
/// Tauri's `emit` needs one payload per listening window.
#[derive(Clone, Serialize)]
struct StatusEvent<'a> {
    server: &'a str,
    status: gateway::Status,
}

/// One sequenced frame, tagged with the server it came from. The frame keeps
/// its generated shape — the envelope around it is what says whose it is.
#[derive(Clone, Serialize)]
struct FrameEvent<'a> {
    server: &'a str,
    frame: &'a ServerFrame,
}

/// Sends what one server's gateway client produces to the WebView.
struct WindowEvents {
    app: AppHandle,
    /// The base URL this connection was opened with. The frontend keys
    /// everything it knows on the same string.
    server: String,
}

impl gateway::Events for WindowEvents {
    fn status(&self, status: gateway::Status) {
        // A failed emit means the window is gone. There is nobody to tell.
        let _ = self.app.emit(
            gateway::STATUS_EVENT,
            StatusEvent {
                server: &self.server,
                status,
            },
        );
    }

    fn frame(&self, frame: &ServerFrame) {
        let _ = self.app.emit(
            gateway::FRAME_EVENT,
            FrameEvent {
                server: &self.server,
                frame,
            },
        );
    }
}

/// Open (or reopen) the connection to one server. Calling this again for the
/// same server replaces its previous connection, which is what makes a frontend
/// reload or a re-sign-in start clean. Other servers are not touched.
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
    let events = WindowEvents {
        app: app.clone(),
        server: base_url.clone(),
    };
    let Some((handle, task)) = gateway::client(&base_url, token, events) else {
        return false;
    };
    tauri::async_runtime::spawn(task);
    connections.with(|held| {
        if let Some(previous) = held.insert(base_url, handle) {
            previous.shutdown();
        }
    });
    true
}

/// Close one server's connection: signing out of it, or removing it.
#[tauri::command]
fn gateway_disconnect(connections: State<'_, Connections>, base_url: String) {
    connections.with(|held| {
        if let Some(handle) = held.remove(&base_url) {
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
    connections.with(|held| {
        held.get(&base_url).is_some_and(|handle| {
            handle.set_token(gateway::Token {
                value: token,
                expires_at_ms,
            })
        })
    })
}

/// Send one client frame to one server. `false` means there was no connection
/// to send it on.
#[tauri::command]
fn gateway_send(connections: State<'_, Connections>, base_url: String, frame: ClientFrame) -> bool {
    connections.with(|held| held.get(&base_url).is_some_and(|handle| handle.send(frame)))
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
        // Signed in-app updates (T-701, ARCHITECTURE §7 baseline 8). Registering
        // the plugin is what makes `[plugins.updater]` readable from Rust; the
        // capability file grants the WebView none of the plugin's own commands,
        // so the page goes through `updates.rs` or not at all.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Connections::default())
        .invoke_handler(tauri::generate_handler![
            activity_probe,
            sessions_load,
            session_save,
            session_forget,
            gateway_connect,
            gateway_disconnect,
            gateway_token,
            gateway_send,
            updates::app_version,
            updates::update_check,
            updates::update_install
        ])
        .run(tauri::generate_context!())
        .expect("failed to start linger");
}
