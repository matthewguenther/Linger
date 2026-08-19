//! Route assembly. Everything REST lives under `/api/v1`; the gateway upgrades
//! at `/api/v1/gateway` (PROTOCOL preamble). Still to mount: `/uploads` and
//! `/media` (M6), `/export` (M9).
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

use axum::routing::any;
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::error::ApiError;
use crate::state::AppState;

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
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn api_not_found() -> ApiError {
    ApiError::not_found("No such thing on this server.")
}
