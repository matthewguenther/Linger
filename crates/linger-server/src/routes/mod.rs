//! Route assembly. Everything REST lives under `/api/v1`; the gateway upgrades
//! at `/api/v1/gateway` (PROTOCOL preamble). Still to mount: `/uploads` and
//! `/media` (M5), `/export` (M8).
//!
//! Unknown paths get the PROTOCOL §1 envelope, not axum's plain-text 404, so
//! the client can always switch on `error.code`.

mod auth;
mod health;
mod invites;
mod messages;
mod rooms;
mod server;
mod setup;
mod users;

use axum::http::{header, HeaderValue, Method};
use axum::routing::any;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::error::ApiError;
use crate::state::AppState;

/// Origins the desktop client runs from. A webview page is a cross-origin
/// caller like any other, so without these the client can't read a single
/// response — which is how this list came to exist (T-301).
///
/// Tauri serves the app from `tauri://localhost`, except on Windows and Android
/// where webview2/webkit need the `tauri.localhost` workaround host. The last
/// entry is Vite's dev server, so `pnpm dev` can talk to a real server.
///
/// It is a fixed list rather than "reflect whatever asked", on purpose: with a
/// wildcard, any web page you happened to visit could read this server's
/// unauthenticated endpoints and learn that it exists. Nothing here needs to be
/// reachable from the open web.
const CLIENT_ORIGINS: [&str; 4] = [
    "tauri://localhost",
    "http://tauri.localhost",
    "https://tauri.localhost",
    "http://localhost:1420",
];

fn cors() -> CorsLayer {
    let origins: Vec<HeaderValue> = CLIENT_ORIGINS
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        // Auth is a bearer token the client holds deliberately. There are no
        // cookies, so there is no ambient authority for a browser to attach.
        .allow_credentials(false)
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .merge(health::router())
        .merge(setup::router())
        .merge(auth::router())
        .merge(users::router())
        .merge(invites::router())
        .merge(server::router())
        .merge(rooms::router())
        .merge(messages::router())
        .route("/gateway", any(crate::gateway::ws_route))
        .fallback(api_not_found);

    Router::new()
        .nest("/api/v1", api)
        .layer(cors())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn api_not_found() -> ApiError {
    ApiError::not_found("No such thing on this server.")
}
