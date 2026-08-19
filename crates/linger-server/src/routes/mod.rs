//! Route assembly. Milestone map:
//!
//! - M1 mounts `/auth`, `/stoop`, `/rooms`, `/messages`, `/users`, `/me`, `/invites`
//! - M2 mounts the gateway at `/api/v1/gateway`
//! - M6 mounts `/uploads` and `/shelf`
//! - M9 mounts `/export`
//!
//! Unknown paths get the PROTOCOL §1 envelope, not axum's plain-text 404, so the
//! client can always switch on `error.code`.

mod health;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::error::ApiError;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let api = Router::new().merge(health::router()).fallback(api_not_found);

    Router::new()
        .nest("/api/v1", api)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn api_not_found() -> ApiError {
    ApiError::not_found("No such thing on this stoop.")
}
